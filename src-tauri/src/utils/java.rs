// Java auto-downloader using Eclipse Adoptium API
// Downloads prebuilt JDK binaries when no suitable Java is found locally.
//
// API: https://api.adoptium.net/v3
// Endpoints:
//   GET /assets/latest/{version}/hotspot?architecture=x64&os=linux&image_type=jdk
//   Returns a JSON array; first element has .binary.package.link (tar.gz URL)
//
// Minecraft Java version requirements:
//   MC 1.20.5+ -> Java 21
//   MC 1.18-1.20.4 -> Java 17
//   MC 1.17 -> Java 16
//   MC <= 1.16 -> Java 8

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

const ADOPTIUM_API: &str = "https://api.adoptium.net/v3";
const USER_AGENT: &str = "OmniLauncherMC/0.1.0";

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AdoptiumAsset {
    binary: AdoptiumBinary,
    release_name: String,
    version: AdoptiumVersion,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AdoptiumBinary {
    package: AdoptiumPackage,
    os: String,
    architecture: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AdoptiumPackage {
    link: String,
    name: String,
    size: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AdoptiumVersion {
    major: u32,
}

/// Get the recommended Java major version for a Minecraft version.
pub fn java_version_for_mc(mc_version: &str) -> u32 {
    let minor: u32 = mc_version
        .split('.')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let patch: u32 = mc_version
        .split('.')
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if minor >= 20 && patch >= 5 {
        21 // MC 1.20.5+
    } else if minor >= 18 {
        17 // MC 1.18 - 1.20.4
    } else if minor == 17 {
        16 // MC 1.17
    } else {
        8 // MC <= 1.16
    }
}

/// Get the path where a Java version should be installed locally.
fn java_install_dir(java_major: u32) -> PathBuf {
    crate::utils::paths::data_dir()
        .join("java")
        .join(format!("jdk-{}", java_major))
}

/// Check if a Java version is already downloaded locally.
pub fn is_java_installed(java_major: u32) -> Option<PathBuf> {
    let dir = java_install_dir(java_major);

    if !dir.exists() {
        return None;
    }

    let java_bin = if cfg!(target_os = "windows") {
        dir.join("bin").join("java.exe")
    } else {
        dir.join("bin").join("java")
    };

    if java_bin.exists() {
        Some(java_bin)
    } else {
        // Try to find the JDK inside a subdirectory (Adoptium extracts to jdk-X.X.X+XX/)
        for entry in std::fs::read_dir(&dir).ok()?.flatten() {
            if entry.file_type().ok()?.is_dir() {
                let candidate = entry
                    .path()
                    .join("bin")
                    .join(if cfg!(target_os = "windows") {
                        "java.exe"
                    } else {
                        "java"
                    });
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
        None
    }
}

/// Download and install a JDK from Adoptium.
/// Returns the path to the java binary.
pub async fn download_java(java_major: u32) -> Result<PathBuf> {
    // Check if already installed
    if let Some(path) = is_java_installed(java_major) {
        return Ok(path);
    }

    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x64"
    };

    let url = format!(
        "{}/assets/latest/{}/hotspot?architecture={}&os={}&image_type=jdk",
        ADOPTIUM_API, java_major, arch, os
    );

    let client = reqwest::Client::new();
    let assets: Vec<AdoptiumAsset> = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to query Adoptium API")?
        .json()
        .await
        .context("Failed to parse Adoptium response")?;

    let asset = assets
        .first()
        .context("No JDK builds available for this platform")?;

    let download_url = &asset.binary.package.link;
    let filename = &asset.binary.package.name;

    log::info!(
        "Downloading JDK {} ({}) from {}",
        java_major,
        filename,
        download_url
    );

    // Download the archive
    let bytes = client
        .get(download_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to download JDK")?
        .bytes()
        .await
        .context("Failed to read JDK download")?;

    // Create install directory
    let install_dir = java_install_dir(java_major);
    std::fs::create_dir_all(&install_dir)?;

    // Write archive to temp file
    let temp_path = install_dir.join(filename);
    std::fs::write(&temp_path, &bytes)?;

    // Extract based on format
    if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        extract_tar_gz(&temp_path, &install_dir)?;
    } else if filename.ends_with(".zip") {
        extract_zip(&temp_path, &install_dir)?;
    } else {
        anyhow::bail!("Unknown archive format: {}", filename);
    }

    // Clean up archive
    let _ = std::fs::remove_file(&temp_path);

    // Find and return the java binary path
    let java_path =
        is_java_installed(java_major).context("JDK downloaded but java binary not found")?;

    log::info!("JDK {} installed at {:?}", java_major, java_path);
    Ok(java_path)
}

/// Find Java: check settings, local install, then system.
/// Auto-downloads if nothing is found.
pub async fn ensure_java(mc_version: &str, custom_path: Option<&str>) -> Result<PathBuf> {
    // 1. Custom path from settings
    if let Some(path) = custom_path {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // 2. Previously auto-downloaded
    let required_major = java_version_for_mc(mc_version);
    if let Some(path) = is_java_installed(required_major) {
        return Ok(path);
    }

    // 3. System Java (via existing launcher.rs detection)
    if let Ok(path) = crate::utils::launcher::find_java(None) {
        // Verify the version is sufficient
        if let Ok(output) = std::process::Command::new(&path).arg("-version").output() {
            let version_str = String::from_utf8_lossy(&output.stderr);
            if check_java_version_sufficient(&version_str, required_major) {
                return Ok(path);
            }
        }
    }

    // 4. Auto-download
    download_java(required_major).await
}

/// Parse java -version output and check if it meets the requirement.
fn check_java_version_sufficient(version_output: &str, required_major: u32) -> bool {
    // java -version output: 'openjdk version "21.0.1" ...'
    // or: 'java version "1.8.0_361" ...'
    if let Some(start) = version_output.find('"') {
        if let Some(end) = version_output[start + 1..].find('"') {
            let version_str = &version_output[start + 1..start + 1 + end];
            let major = if version_str.starts_with("1.") {
                // Old format: 1.8.0_xxx -> major = 8
                version_str
                    .split('.')
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0)
            } else {
                // New format: 21.0.1 -> major = 21
                version_str
                    .split('.')
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0)
            };
            return major >= required_major;
        }
    }
    false
}

/// Extract a .tar.gz archive to a destination directory.
fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<()> {
    use std::process::Command;

    // Use system tar for extraction (available on all platforms we support)
    let status = Command::new("tar")
        .args([
            "xzf",
            &archive_path.to_string_lossy(),
            "-C",
            &dest.to_string_lossy(),
            "--strip-components=0",
        ])
        .status()
        .context("Failed to run tar (is tar installed?)")?;

    if !status.success() {
        anyhow::bail!("tar extraction failed with status: {}", status);
    }

    Ok(())
}

/// Extract a .zip archive to a destination directory.
fn extract_zip(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let outpath = dest.join(entry.mangled_name());

        if entry.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buf)?;
            std::fs::write(&outpath, &buf)?;
        }
    }

    Ok(())
}
