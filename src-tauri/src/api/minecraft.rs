use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const USER_AGENT: &str = "OmniLauncherMC/0.1.0 (github.com/OmniLauncherMC)";

/// Get an HTTP client: use the shared one if provided, otherwise create a new one.
fn get_client(client: Option<&reqwest::Client>) -> reqwest::Client {
    client.cloned().unwrap_or_else(|| {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client")
    })
}

// ── Types ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionManifestEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDetails {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    pub arguments: Option<VersionArguments>,
    pub libraries: Vec<Library>,
    pub downloads: VersionDownloads,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionArguments {
    pub game: Vec<serde_json::Value>,
    pub jvm: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<LibraryRule>>,
    pub natives: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    pub classifiers: Option<std::collections::HashMap<String, LibraryArtifact>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryArtifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryRule {
    pub action: String,
    pub os: Option<OsRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDownloads {
    pub client: DownloadInfo,
    pub server: Option<DownloadInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
    #[serde(rename = "totalSize")]
    pub total_size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndexData {
    pub objects: std::collections::HashMap<String, AssetObject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

// ── API Functions ──────────────────────────────────────────────

/// Fetch the Minecraft version manifest from Mojang.
pub async fn fetch_version_manifest(client: Option<&reqwest::Client>) -> Result<VersionManifest> {
    let client = get_client(client);
    let manifest: VersionManifest = client
        .get(VERSION_MANIFEST_URL)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch version manifest")?
        .json()
        .await
        .context("Failed to parse version manifest")?;
    Ok(manifest)
}

/// Fetch details for a specific version (by URL from the manifest).
pub async fn fetch_version_details(client: Option<&reqwest::Client>, version_url: &str) -> Result<VersionDetails> {
    let client = get_client(client);
    let details: VersionDetails = client
        .get(version_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch version details")?
        .json()
        .await
        .context("Failed to parse version details")?;
    Ok(details)
}

/// Fetch an asset index.
pub async fn fetch_asset_index(client: Option<&reqwest::Client>, index_url: &str) -> Result<AssetIndexData> {
    let client = get_client(client);
    let index: AssetIndexData = client
        .get(index_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch asset index")?
        .json()
        .await
        .context("Failed to parse asset index")?;
    Ok(index)
}

/// Download a file from a URL to a local path, creating parent directories.
pub async fn download_file(client: Option<&reqwest::Client>, url: &str, dest: &Path) -> Result<()> {
    if dest.exists() {
        return Ok(()); // Already downloaded
    }

    let client = get_client(client);
    let resp = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .with_context(|| format!("Failed to download {}", url))?;

    if !resp.status().is_success() {
        anyhow::bail!("Download failed ({}): {}", resp.status(), url);
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = resp.bytes().await?;
    std::fs::write(dest, &bytes)?;
    Ok(())
}

/// Download a file with SHA1 verification.
pub async fn download_file_verified(
    client: Option<&reqwest::Client>,
    url: &str,
    dest: &Path,
    expected_sha1: &str,
) -> Result<()> {
    use sha1::{Digest, Sha1};

    download_file(client, url, dest).await?;

    // Verify hash
    let data = std::fs::read(dest)?;
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let actual = hex::encode(hasher.finalize());

    if actual != expected_sha1 {
        std::fs::remove_file(dest)?;
        anyhow::bail!(
            "SHA1 mismatch for {}: expected {}, got {}",
            dest.display(),
            expected_sha1,
            actual
        );
    }

    Ok(())
}

/// Check if a library rule set allows the current platform.
pub fn rules_allow(rules: &Option<Vec<LibraryRule>>) -> bool {
    match rules {
        None => true, // No rules = always allow
        Some(rules) => {
            let mut allowed = false;
            for rule in rules {
                match rule.action.as_str() {
                    "allow" => {
                        if let Some(os) = &rule.os {
                            if os.name.as_deref() == Some(std::env::consts::OS) {
                                allowed = true;
                            }
                        } else {
                            allowed = true; // No OS filter = allow on all
                        }
                    }
                    "disallow" => {
                        if let Some(os) = &rule.os {
                            if os.name.as_deref() == Some(std::env::consts::OS) {
                                allowed = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
            allowed
        }
    }
}
