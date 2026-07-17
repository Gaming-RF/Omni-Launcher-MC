// Java installation manager — detection, version parsing, per-instance selection, auto-download.
//
// Architecture:
// - On startup: scan well-known paths + PATH + JAVA_HOME for all Java installations
// - Parse `java -version` stderr for each to get major version, vendor, arch
// - Store in SQLite `java_installations` table
// - Per-instance: auto-select best Java for game version OR use manual override
// - Auto-download: fetch Adoptium Temurin binaries when no suitable Java exists

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tokio::process::Command as TokioCommand;

/// A detected or downloaded Java installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaInstallation {
    pub id: String,
    pub path: PathBuf,
    pub major_version: u32,
    pub arch: String,
    pub vendor: String,
    pub is_auto_downloaded: bool,
}

/// Java version info parsed from `java -version` stderr.
#[derive(Debug, Clone)]
pub struct JavaVersionInfo {
    pub major_version: u32,
    pub full_version: String,
    pub vendor: String,
    pub arch: String,
}

// ── Detection ─────────────────────────────────────────────────

/// Scan the system for all Java installations.
/// Returns a list of detected Java installations with parsed version info.
pub async fn detect_all_javas() -> Vec<JavaInstallation> {
    let mut candidates = Vec::new();

    // 1. JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let bin = java_bin_path(&PathBuf::from(&java_home));
        if bin.exists() {
            candidates.push(bin);
        }
    }

    // 2. PATH via `which -a java` (Linux/macOS) or `where java` (Windows)
    candidates.extend(find_java_on_path());

    // 3. Platform-specific well-known paths
    candidates.extend(well_known_java_paths());

    // 4. SDKMAN installations
    if let Some(home) = dirs::home_dir() {
        let sdkman_dir = home.join(".sdkman").join("candidates").join("java");
        if sdkman_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&sdkman_dir) {
                for entry in entries.flatten() {
                    let bin = java_bin_path(&entry.path());
                    if bin.exists() {
                        candidates.push(bin);
                    }
                }
            }
        }
    }

    // Deduplicate by canonical path
    candidates.sort();
    candidates.dedup();

    // Parse version info for each candidate concurrently
    let mut installations = Vec::new();
    for path in candidates {
        if let Some(info) = parse_java_version(&path).await {
            installations.push(JavaInstallation {
                id: format!("{}-{}-{}", info.major_version, info.vendor, info.arch),
                path,
                major_version: info.major_version,
                arch: info.arch,
                vendor: info.vendor,
                is_auto_downloaded: false,
            });
        }
    }

    // Deduplicate by major_version + vendor (keep shortest path)
    installations.sort_by(|a, b| {
        a.major_version
            .cmp(&b.major_version)
            .then(a.path.cmp(&b.path))
    });
    installations.dedup_by(|a, b| a.major_version == b.major_version && a.vendor == b.vendor);

    installations
}

/// Find the best Java for a given Minecraft version.
/// Minecraft 1.17+ requires Java 16+, 1.18+ requires Java 17+, 1.20.5+ requires Java 21+.
pub fn best_java_for_mc(
    installations: &[JavaInstallation],
    mc_version: &str,
) -> Option<&JavaInstallation> {
    let required = mc_required_java(mc_version);

    // Prefer exact match, then lowest version that meets requirement
    let mut eligible: Vec<_> = installations
        .iter()
        .filter(|j| j.major_version >= required)
        .collect();

    if eligible.is_empty() {
        return None;
    }

    // Sort by: prefer non-auto-downloaded, then closest version match
    eligible.sort_by(|a, b| {
        a.is_auto_downloaded
            .cmp(&b.is_auto_downloaded)
            .then_with(|| a.major_version.cmp(&b.major_version))
    });

    eligible.first().map(|&j| j)
}

/// Determine the minimum Java major version required for a Minecraft version.
fn mc_required_java(mc_version: &str) -> u32 {
    let parts: Vec<&str> = mc_version.split('.').collect();
    let major: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    match (major, minor) {
        (1, v) if v >= 20 => {
            // 1.20.5+ requires Java 21
            if v > 20 || (v == 20 && mc_version.contains("5") || mc_version.contains("6")) {
                21
            } else {
                17
            }
        }
        (1, 19..=20) => 17, // 1.18-1.20.4: Java 17
        (1, 17..=18) => 16, // 1.17-1.17.1: Java 16
        (1, v) if v >= 13 => 8, // 1.13-1.16.5: Java 8
        _ => 8,             // Older versions: Java 8
    }
}

// ── Version Parsing ───────────────────────────────────────────

/// Parse `java -version` stderr to extract version info.
async fn parse_java_version(java_path: &Path) -> Option<JavaVersionInfo> {
    let output = TokioCommand::new(java_path)
        .arg("-version")
        .output()
        .await
        .ok()?;

    // java -version outputs to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() && stderr.is_empty() {
        return None;
    }

    parse_java_version_string(&stderr, java_path)
}

/// Parse the version string from `java -version` output.
fn parse_java_version_string(output: &str, java_path: &Path) -> Option<JavaVersionInfo> {
    // Example outputs:
    //   openjdk version "21.0.3" 2024-04-16 LTS
    //   java version "17.0.10" 2024-01-16 LTS
    //   openjdk version "1.8.0_402"

    let version_line = output.lines().next()?;
    let first_quote = version_line.find('"')?;
    let rest = &version_line[first_quote + 1..];
    let second_quote = rest.find('"')?;
    let version_str = &rest[..second_quote];

    let major = if version_str.starts_with("1.") {
        // Old format: "1.8.0_xxx" → major = 8
        version_str
            .split('.')
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(8)
    } else {
        // New format: "21.0.3" → major = 21
        version_str
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    };

    let vendor = detect_vendor(output, java_path);
    let arch = detect_arch(java_path);

    Some(JavaVersionInfo {
        major_version: major,
        full_version: version_str.to_string(),
        vendor,
        arch,
    })
}

/// Detect the Java vendor from version output or path.
fn detect_vendor(output: &str, path: &Path) -> String {
    let lower = output.to_lowercase();
    let path_str = path.to_string_lossy().to_lowercase();

    if lower.contains("openjdk") || lower.contains("temurin") || lower.contains("adoptium") {
        "Adoptium/Temurin".into()
    } else if lower.contains("graalvm") {
        "GraalVM".into()
    } else if lower.contains("corretto") || path_str.contains("corretto") {
        "Amazon Corretto".into()
    } else if lower.contains("zulu") || path_str.contains("zulu") {
        "Azul Zulu".into()
    } else if lower.contains("microsoft") || path_str.contains("microsoft") {
        "Microsoft OpenJDK".into()
    } else if lower.contains("semeru") || path_str.contains("semeru") {
        "IBM Semeru".into()
    } else if lower.contains("oracle") {
        "Oracle".into()
    } else {
        "Unknown".into()
    }
}

/// Detect architecture from java binary path or system.
fn detect_arch(java_path: &Path) -> String {
    let path_str = java_path.to_string_lossy().to_lowercase();

    if path_str.contains("aarch64") || path_str.contains("arm64") {
        "aarch64".into()
    } else if path_str.contains("x86_64") || path_str.contains("amd64") || path_str.contains("x64")
    {
        "x86_64".into()
    } else if path_str.contains("x86") || path_str.contains("i686") || path_str.contains("i386") {
        "x86".into()
    } else {
        std::env::consts::ARCH.to_string()
    }
}

// ── Path Discovery Helpers ────────────────────────────────────

/// Construct the path to the java binary inside a JAVA_HOME-like directory.
fn java_bin_path(home: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        home.join("bin").join("java.exe")
    } else {
        home.join("bin").join("java")
    }
}

/// Find java on PATH using `which -a` (Unix) or `where` (Windows).
fn find_java_on_path() -> Vec<PathBuf> {
    let (cmd, arg) = if cfg!(target_os = "windows") {
        ("where", "java")
    } else {
        ("which", "-a")
    };

    let mut paths = Vec::new();

    // For `which -a java`, pass "java" as the target
    let output = if cfg!(target_os = "windows") {
        StdCommand::new(cmd).arg(arg).output()
    } else {
        StdCommand::new(cmd).arg(arg).arg("java").output()
    };

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let path = PathBuf::from(line.trim());
            if path.exists() {
                paths.push(path);
            }
        }
    }

    paths
}

/// Platform-specific well-known Java installation paths.
fn well_known_java_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if cfg!(target_os = "windows") {
        // Windows: Program Files
        let bases = [
            r"C:\Program Files\Java",
            r"C:\Program Files\Eclipse Adoptium",
            r"C:\Program Files\Microsoft",
            r"C:\Program Files\Amazon Corretto",
            r"C:\Program Files\Azul\Zulu",
            r"C:\Program Files\GraalVM",
        ];
        for base in &bases {
            if let Ok(entries) = std::fs::read_dir(base) {
                for entry in entries.flatten() {
                    let bin = entry.path().join("bin").join("java.exe");
                    if bin.exists() {
                        paths.push(bin);
                    }
                }
            }
        }
    } else if cfg!(target_os = "macos") {
        // macOS: JavaVirtualMachines + Homebrew
        let jvm_dir = PathBuf::from("/Library/Java/JavaVirtualMachines");
        if let Ok(entries) = std::fs::read_dir(&jvm_dir) {
            for entry in entries.flatten() {
                let bin = entry
                    .path()
                    .join("Contents")
                    .join("Home")
                    .join("bin")
                    .join("java");
                if bin.exists() {
                    paths.push(bin);
                }
            }
        }
        // Homebrew
        for prefix in &["/opt/homebrew/opt", "/usr/local/opt"] {
            for version in &["@21", "@17", "@16", "@11", "@8"] {
                let bin = PathBuf::from(prefix)
                    .join(format!("openjdk{}", version))
                    .join("bin")
                    .join("java");
                if bin.exists() {
                    paths.push(bin);
                }
            }
        }
    } else {
        // Linux: /usr/lib/jvm
        let jvm_dir = PathBuf::from("/usr/lib/jvm");
        if let Ok(entries) = std::fs::read_dir(&jvm_dir) {
            for entry in entries.flatten() {
                let bin = entry.path().join("bin").join("java");
                if bin.exists() {
                    paths.push(bin);
                }
            }
        }
        // Flatpak, Snap, etc.
        let extra = ["/usr/bin/java", "/snap/java/current/bin/java"];
        for p in &extra {
            let bin = PathBuf::from(p);
            if bin.exists() {
                paths.push(bin);
            }
        }
    }

    paths
}

// ── Auto-Download (Adoptium Temurin) ──────────────────────────

/// Auto-download a JDK from Adoptium Temurin.
/// Returns the path to the extracted java binary.
pub async fn auto_download_java(
    major_version: u32,
    dest_dir: &Path,
) -> Result<JavaInstallation> {
    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "mac",
        "windows" => "windows",
        _ => anyhow::bail!("Unsupported OS for Java auto-download"),
    };

    let arch = match std::env::consts::ARCH {
        "x86_64" | "amd64" => "x64",
        "aarch64" | "arm64" => "aarch64",
        _ => anyhow::bail!("Unsupported architecture for Java auto-download"),
    };

    let image_type = if cfg!(target_os = "windows") {
        "jdk" // Windows: need JDK for jpackage/native access
    } else {
        "jre" // Unix: JRE suffices for running Minecraft
    };

    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/{}/hotspot/normal/eclipse",
        major_version, os, arch, image_type
    );

    log::info!("Downloading Temurin JRE {} for {}/{}", major_version, os, arch);

    // Download the archive
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "OmniLauncherMC/0.1.0")
        .send()
        .await
        .context("Failed to download Temurin JRE")?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "Temurin download failed ({}). Check if Java {} is available for {}/{}",
            resp.status(),
            major_version,
            os,
            arch
        );
    }

    let bytes = resp.bytes().await?;
    let dest = dest_dir.join(format!("temurin-{}", major_version));
    std::fs::create_dir_all(&dest)?;

    // Extract based on format
    let archive_name = if cfg!(target_os = "windows") {
        "jdk.zip"
    } else {
        "jdk.tar.gz"
    };
    let archive_path = dest.join(archive_name);
    std::fs::write(&archive_path, &bytes)?;

    // Extract with tar (cross-platform)
    let status = TokioCommand::new("tar")
        .args(["xf", &archive_path.to_string_lossy()])
        .current_dir(&dest)
        .status()
        .await
        .context("Failed to extract JDK archive")?;

    if !status.success() {
        anyhow::bail!("Failed to extract JDK archive");
    }

    // Clean up archive
    let _ = tokio::fs::remove_file(&archive_path).await;

    // Find the extracted java binary
    let java_bin = find_extracted_java(&dest)?;

    Ok(JavaInstallation {
        id: format!("temurin-{}-auto", major_version),
        path: java_bin,
        major_version,
        arch: arch.to_string(),
        vendor: "Adoptium/Temurin (auto)".to_string(),
        is_auto_downloaded: true,
    })
}

/// Find the java binary inside an extracted JDK directory.
fn find_extracted_java(extract_dir: &Path) -> Result<PathBuf> {
    // Look for bin/java in subdirectories
    if let Ok(entries) = std::fs::read_dir(extract_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let bin = java_bin_path(&path);
                if bin.exists() {
                    return Ok(bin);
                }
            }
        }
    }

    // Fallback: search recursively (shallow)
    let bin_name = if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    };

    for entry in walk_dir(extract_dir, 3) {
        if entry.file_name().map_or(false, |f| f == bin_name) {
            return Ok(entry);
        }
    }

    anyhow::bail!("Could not find java binary in extracted JDK")
}

/// Shallow recursive directory walk with depth limit.
fn walk_dir(dir: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    walk_dir_inner(dir, 0, max_depth, &mut results);
    results
}

fn walk_dir_inner(dir: &Path, depth: usize, max_depth: usize, results: &mut Vec<PathBuf>) {
    if depth > max_depth {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            results.push(path.clone());
            if path.is_dir() {
                walk_dir_inner(&path, depth + 1, max_depth, results);
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_required_java() {
        assert_eq!(mc_required_java("1.12.2"), 8);
        assert_eq!(mc_required_java("1.16.5"), 8);
        assert_eq!(mc_required_java("1.17.1"), 16);
        assert_eq!(mc_required_java("1.18.2"), 17);
        assert_eq!(mc_required_java("1.19.4"), 17);
        assert_eq!(mc_required_java("1.20.1"), 17);
        assert_eq!(mc_required_java("1.21.4"), 21);
    }

    #[test]
    fn test_parse_version_string() {
        let output = r#"openjdk version "21.0.3" 2024-04-16 LTS
OpenJDK Runtime Environment Temurin-21.0.3+9 (build 21.0.3+9-LTS)
OpenJDK 64-Bit Server VM Temurin-21.0.3+9 (build 21.0.3+9-LTS, mixed mode, sharing)"#;

        let info = parse_java_version_string(output, Path::new("/usr/lib/jvm/temurin-21/bin/java"));
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.major_version, 21);
        assert_eq!(info.full_version, "21.0.3");
        assert_eq!(info.vendor, "Adoptium/Temurin");
    }

    #[test]
    fn test_parse_old_version_string() {
        let output = r#"java version "1.8.0_402"
Java(TM) SE Runtime Environment (build 1.8.0_402-b06)
Java HotSpot(TM) 64-Bit Server VM (build 25.402-b06, mixed mode)"#;

        let info =
            parse_java_version_string(output, Path::new("/usr/lib/jvm/java-8-oracle/bin/java"));
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.major_version, 8);
        assert_eq!(info.full_version, "1.8.0_402");
        assert_eq!(info.vendor, "Oracle");
    }
}
