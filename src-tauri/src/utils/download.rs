// Download engine with concurrent batch support, hash verification, resume, and progress events.
//
// Architecture:
// - Shared reqwest::Client (connection pooling) passed via Tauri AppState
// - Semaphore-bounded concurrency (default 8 parallel downloads)
// - SHA1/SHA-256/SHA-512/CRC32 hash verification
// - HTTP Range-based resume for interrupted downloads
// - Progress reporting via Tauri events ("download-progress")
// - Skip-if-exists with hash check

use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha1::Digest as Sha1Digest;
use sha2::Digest as Sha256Digest;
use sha2::Sha512 as Sha512Hasher;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

/// Hash algorithm to use for verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HashAlgo {
    Sha1,
    Sha256,
    Sha512,
    Crc32,
    Md5,
}

impl HashAlgo {
    /// Parse from CurseForge's algo integer (1=sha1, 2=md5, etc.)
    pub fn from_curseforge_id(id: i32) -> Self {
        match id {
            1 => HashAlgo::Sha1,
            2 => HashAlgo::Md5,
            // CurseForge doesn't define further algo IDs commonly
            _ => HashAlgo::Sha1,
        }
    }
}

/// A single file to download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    /// Remote URL to download from.
    pub url: String,
    /// Local destination path (full file path).
    pub dest: PathBuf,
    /// Expected hash for verification. If None, skip verification.
    pub expected_hash: Option<String>,
    /// Hash algorithm. Defaults to SHA1 if not specified.
    pub hash_algo: HashAlgo,
    /// Human-readable name for progress display (e.g. "Sodium 0.6.0").
    pub display_name: String,
}

impl DownloadTask {
    pub fn new(url: impl Into<String>, dest: impl Into<PathBuf>) -> Self {
        Self {
            url: url.into(),
            dest: dest.into(),
            expected_hash: None,
            hash_algo: HashAlgo::Sha1,
            display_name: String::new(),
        }
    }

    pub fn with_hash(mut self, hash: impl Into<String>, algo: HashAlgo) -> Self {
        self.expected_hash = Some(hash.into());
        self.hash_algo = algo;
        self
    }

    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }
}

/// Progress event emitted to the frontend via Tauri.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// Index of this task in the batch.
    pub index: usize,
    /// Total tasks in the batch.
    pub total: usize,
    /// Bytes downloaded so far for this file.
    pub bytes_downloaded: u64,
    /// Total bytes for this file (0 if unknown).
    pub total_bytes: u64,
    /// Display name of the file being downloaded.
    pub display_name: String,
    /// Status of this download.
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Downloading,
    Verifying,
    Completed,
    Skipped,
    Failed(String),
}

/// Result of a single download task.
#[derive(Debug)]
pub struct DownloadResult {
    pub index: usize,
    pub success: bool,
    pub bytes: u64,
    pub skipped: bool,
    pub error: Option<String>,
}

/// Batch download summary.
#[derive(Debug, Serialize)]
pub struct BatchSummary {
    pub total: usize,
    pub downloaded: usize,
    pub skipped: usize,
    pub failed: usize,
    pub total_bytes: u64,
    pub errors: Vec<String>,
}

/// Configuration for the download engine.
pub struct DownloadConfig {
    /// Maximum concurrent downloads.
    pub max_concurrency: usize,
    /// Connection timeout in seconds.
    pub timeout_secs: u64,
    /// User-Agent string.
    pub user_agent: String,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 8,
            timeout_secs: 30,
            user_agent: "OmniLauncherMC/0.1.0 (github.com/OmniLauncherMC)".into(),
        }
    }
}

/// The download engine. Use a single instance across the application.
pub struct DownloadEngine {
    client: reqwest::Client,
    config: DownloadConfig,
}

impl DownloadEngine {
    /// Create a new download engine with a pre-built client.
    pub fn new(client: reqwest::Client, config: DownloadConfig) -> Self {
        Self { client, config }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        let config = DownloadConfig::default();
        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .pool_max_idle_per_host(config.max_concurrency)
            .build()
            .expect("Failed to build HTTP client");
        Self::new(client, config)
    }

    /// Download a single file. Returns bytes downloaded (0 if skipped).
    pub async fn download_one(&self, task: &DownloadTask) -> Result<DownloadResult> {
        self.download_one_with_progress(task, None, 0, 1).await
    }

    /// Download a single file with progress callback.
    /// The callback receives DownloadProgress updates.
    async fn download_one_with_progress(
        &self,
        task: &DownloadTask,
        app_handle: Option<&tauri::AppHandle>,
        index: usize,
        total: usize,
    ) -> Result<DownloadResult> {
        let progress = |status: DownloadStatus, bytes: u64, total_bytes: u64| {
            DownloadProgress {
                index,
                total,
                bytes_downloaded: bytes,
                total_bytes,
                display_name: task.display_name.clone(),
                status,
            }
        };

        // Skip if file exists and hash matches
        if task.dest.exists() {
            if let Some(ref expected) = task.expected_hash {
                if verify_hash(&task.dest, expected, &task.hash_algo).is_ok() {
                    emit_progress(app_handle, &progress(DownloadStatus::Skipped, 0, 0));
                    return Ok(DownloadResult {
                        index,
                        success: true,
                        bytes: 0,
                        skipped: true,
                        error: None,
                    });
                }
            } else {
                // No hash to check, file exists — skip
                emit_progress(app_handle, &progress(DownloadStatus::Skipped, 0, 0));
                return Ok(DownloadResult {
                    index,
                    success: true,
                    bytes: 0,
                    skipped: true,
                    error: None,
                });
            }
            // Hash mismatch — re-download
            tokio::fs::remove_file(&task.dest).await.ok();
        }

        // Create parent directories
        if let Some(parent) = task.dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Check for partial download to support resume
        let existing_bytes = if task.dest.exists() {
            tokio::fs::metadata(&task.dest).await?.len()
        } else {
            0
        };

        emit_progress(
            app_handle,
            &progress(DownloadStatus::Downloading, 0, 0),
        );

        // Build request with optional Range header for resume
        let mut req = self.client.get(&task.url);
        if existing_bytes > 0 {
            req = req.header("Range", format!("bytes={}-", existing_bytes));
        }

        let resp = req.send().await.with_context(|| {
            format!("Failed to download {}", task.url)
        })?;

        let status = resp.status();

        // Handle server responses
        let mut downloaded: u64 = existing_bytes;
        let content_length = resp
            .content_length()
            .unwrap_or(0);
        let total_bytes = if status == reqwest::StatusCode::PARTIAL_CONTENT {
            existing_bytes + content_length
        } else if status.is_success() {
            // Server doesn't support range or returned full content
            downloaded = 0;
            content_length
        } else {
            anyhow::bail!("Download failed ({}): {}", status, task.url);
        };

        // Open file for writing (append if resuming, create if not)
        let mut file = if status == reqwest::StatusCode::PARTIAL_CONTENT && existing_bytes > 0 {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(&task.dest)
                .await?
        } else {
            tokio::fs::File::create(&task.dest).await?
        };

        // Stream download
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error reading download stream")?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // Emit progress every ~256KB to avoid flooding
            if downloaded % (256 * 1024) < chunk.len() as u64
                || downloaded == total_bytes
            {
                emit_progress(
                    app_handle,
                    &progress(DownloadStatus::Downloading, downloaded, total_bytes),
                );
            }
        }
        file.flush().await?;

        // Verify hash if provided
        if let Some(ref expected) = task.expected_hash {
            emit_progress(app_handle, &progress(DownloadStatus::Verifying, downloaded, total_bytes));
            if let Err(e) = verify_hash(&task.dest, expected, &task.hash_algo) {
                tokio::fs::remove_file(&task.dest).await.ok();
                emit_progress(
                    app_handle,
                    &progress(DownloadStatus::Failed(e.to_string()), downloaded, total_bytes),
                );
                return Ok(DownloadResult {
                    index,
                    success: false,
                    bytes: downloaded,
                    skipped: false,
                    error: Some(e.to_string()),
                });
            }
        }

        emit_progress(app_handle, &progress(DownloadStatus::Completed, downloaded, total_bytes));

        Ok(DownloadResult {
            index,
            success: true,
            bytes: downloaded,
            skipped: false,
            error: None,
        })
    }

    /// Download a batch of files concurrently.
    /// Returns a summary of the batch operation.
    pub async fn download_batch(
        &self,
        tasks: &[DownloadTask],
        app_handle: Option<&tauri::AppHandle>,
    ) -> BatchSummary {
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));
        let total = tasks.len();
        let mut handles = Vec::with_capacity(total);

        for (i, task) in tasks.iter().enumerate() {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let client = self.client.clone();
            let config_timeout = self.config.timeout_secs;
            let task = task.clone();
            let handle = app_handle.cloned();

            handles.push(tokio::spawn(async move {
                let engine = DownloadEngine::new(
                    client,
                    DownloadConfig {
                        max_concurrency: 1,
                        timeout_secs: config_timeout,
                        user_agent: String::new(), // Not used in sub-call
                    },
                );
                let result = engine
                    .download_one_with_progress(&task, handle.as_ref(), i, total)
                    .await;
                drop(permit); // Release semaphore slot
                (i, result)
            }));
        }

        let mut summary = BatchSummary {
            total,
            downloaded: 0,
            skipped: 0,
            failed: 0,
            total_bytes: 0,
            errors: Vec::new(),
        };

        for handle in handles {
            match handle.await {
                Ok((_, Ok(result))) => {
                    summary.total_bytes += result.bytes;
                    if result.skipped {
                        summary.skipped += 1;
                    } else if result.success {
                        summary.downloaded += 1;
                    } else {
                        summary.failed += 1;
                        if let Some(err) = result.error {
                            summary.errors.push(err);
                        }
                    }
                }
                Ok((i, Err(e))) => {
                    summary.failed += 1;
                    summary.errors.push(format!("Task {}: {}", i, e));
                }
                Err(e) => {
                    summary.failed += 1;
                    summary.errors.push(format!("Join error: {}", e));
                }
            }
        }

        summary
    }

    /// Get a reference to the underlying HTTP client.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

// ── Hash verification ─────────────────────────────────────────

/// Verify a file's hash against an expected value.
pub fn verify_hash(path: &Path, expected: &str, algo: &HashAlgo) -> Result<()> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read {} for hash verification", path.display()))?;

    let actual = match algo {
        HashAlgo::Sha1 => {
            let mut hasher = sha1::Sha1::new();
            hasher.update(&data);
            hex::encode(hasher.finalize())
        }
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            hasher.update(&data);
            hex::encode(hasher.finalize())
        }
        HashAlgo::Sha512 => {
            let mut hasher = Sha512Hasher::new();
            hasher.update(&data);
            hex::encode(hasher.finalize())
        }
        HashAlgo::Crc32 => {
            let crc = crc32fast::hash(&data);
            format!("{:08x}", crc)
        }
        HashAlgo::Md5 => {
            let mut hasher = md5::compute(&data);
            format!("{:x}", hasher)
        }
    };

    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        anyhow::bail!(
            "Hash mismatch for {}: expected {} ({}), got {}",
            path.display(),
            expected,
            algo_name(algo),
            actual
        )
    }
}

fn algo_name(algo: &HashAlgo) -> &'static str {
    match algo {
        HashAlgo::Sha1 => "SHA1",
        HashAlgo::Sha256 => "SHA256",
        HashAlgo::Sha512 => "SHA512",
        HashAlgo::Crc32 => "CRC32",
        HashAlgo::Md5 => "MD5",
    }
}

// ── Progress event emission ───────────────────────────────────

fn emit_progress(app_handle: Option<&tauri::AppHandle>, progress: &DownloadProgress) {
    if let Some(handle) = app_handle {
        let _ = handle.emit("download-progress", progress);
    }
}
