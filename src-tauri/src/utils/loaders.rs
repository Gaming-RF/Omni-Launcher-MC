// Mod loader installer — Fabric, Forge, NeoForge, Quilt.
//
// Architecture:
// - Fetches version metadata from each loader's API
// - Downloads loader installer JARs
// - Generates merged version profiles (vanilla + loader)
// - Writes to the launcher's versions directory
//
// Flow: User selects loader → fetch metadata → download installer → run or merge profile → instance ready

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const USER_AGENT: &str = "OmniLauncherMC/0.1.0 (github.com/OmniLauncherMC)";

/// Supported mod loaders.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModLoader {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
    Quilt,
}

impl ModLoader {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModLoader::Vanilla => "vanilla",
            ModLoader::Fabric => "fabric",
            ModLoader::Forge => "forge",
            ModLoader::NeoForge => "neoforge",
            ModLoader::Quilt => "quilt",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fabric" => ModLoader::Fabric,
            "forge" => ModLoader::Forge,
            "neoforge" => ModLoader::NeoForge,
            "quilt" => ModLoader::Quilt,
            _ => ModLoader::Vanilla,
        }
    }
}

/// A loader version available for installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    pub loader: ModLoader,
    pub version: String,
    pub stable: bool,
    pub game_versions: Vec<String>,
}

// ── Fabric ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FabricLoaderVersion {
    separator: Option<String>,
    build: u32,
    maven: String,
    version: String,
    stable: bool,
}

#[derive(Deserialize)]
struct FabricGameVersion {
    version: String,
    stable: bool,
}

/// Fetch available Fabric loader versions for a game version.
pub async fn fetch_fabric_loaders(game_version: &str) -> Result<Vec<LoaderVersion>> {
    let client = reqwest::Client::new();

    // Get loader versions
    let loaders: Vec<FabricLoaderVersion> = client
        .get("https://meta.fabricmc.net/v2/versions/loader")
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch Fabric loader versions")?
        .json()
        .await
        .context("Failed to parse Fabric loader versions")?;

    Ok(loaders
        .into_iter()
        .map(|l| LoaderVersion {
            loader: ModLoader::Fabric,
            version: l.version,
            stable: l.stable,
            game_versions: vec![game_version.to_string()],
        })
        .collect())
}

/// Fetch the Fabric version profile JSON for a specific game + loader combination.
/// This is the merged version profile that can be written to the versions directory.
pub async fn fetch_fabric_profile(game_version: &str, loader_version: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
        game_version, loader_version
    );

    let profile: serde_json::Value = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch Fabric profile")?
        .json()
        .await
        .context("Failed to parse Fabric profile")?;

    Ok(profile)
}

/// Install Fabric for an instance.
/// Downloads the version profile and writes it to the versions directory.
pub async fn install_fabric(
    game_version: &str,
    loader_version: &str,
    versions_dir: &Path,
) -> Result<PathBuf> {
    let profile = fetch_fabric_profile(game_version, loader_version).await?;

    let profile_id = format!("fabric-loader-{}-{}", loader_version, game_version);
    let version_dir = versions_dir.join(&profile_id);
    std::fs::create_dir_all(&version_dir)?;

    let profile_path = version_dir.join(format!("{}.json", profile_id));
    let json = serde_json::to_string_pretty(&profile)?;
    std::fs::write(&profile_path, json)?;

    log::info!("Installed Fabric {} for MC {}", loader_version, game_version);
    Ok(profile_path)
}

// ── Forge ─────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ForgePromotions {
    // Key format: "1.20.1-47.2.0" etc.
    #[serde(flatten)]
    versions: std::collections::HashMap<String, String>,
}

/// Fetch available Forge versions.
/// Forge uses a "promotions" endpoint that maps game versions to recommended Forge versions.
pub async fn fetch_forge_versions() -> Result<Vec<LoaderVersion>> {
    let client = reqwest::Client::new();
    let promotions: ForgePromotions = client
        .get("https://files.minecraftforge.net/maven/net/minecraftforge/forge/promotions_slim.json")
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch Forge promotions")?
        .json()
        .await
        .context("Failed to parse Forge promotions")?;

    let mut versions = Vec::new();
    for (key, forge_version) in &promotions.versions {
        // Key format: "1.20.1-47.2.0" where the first part is game version
        if let Some(dash_pos) = key.find('-') {
            let game_version = &key[..dash_pos];
            versions.push(LoaderVersion {
                loader: ModLoader::Forge,
                version: forge_version.clone(),
                stable: true,
                game_versions: vec![game_version.to_string()],
            });
        }
    }

    Ok(versions)
}

/// Get the Forge installer download URL for a specific version.
pub fn forge_installer_url(game_version: &str, forge_version: &str) -> String {
    format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
        game_version, forge_version, game_version, forge_version
    )
}

/// Install Forge for an instance.
/// Downloads the installer JAR and runs it headlessly to generate the version profile.
pub async fn install_forge(
    game_version: &str,
    forge_version: &str,
    versions_dir: &Path,
    java_path: &Path,
) -> Result<PathBuf> {
    let installer_url = forge_installer_url(game_version, forge_version);
    let profile_id = format!("forge-{}-{}", game_version, forge_version);
    let version_dir = versions_dir.join(&profile_id);

    // Check if already installed
    let profile_json = version_dir.join(format!("{}.json", profile_id));
    if profile_json.exists() {
        return Ok(profile_json);
    }

    // Download installer
    let temp_dir = std::env::temp_dir().join("omnilauncher-forge");
    std::fs::create_dir_all(&temp_dir)?;
    let installer_path = temp_dir.join(format!("forge-{}-{}-installer.jar", game_version, forge_version));

    let client = reqwest::Client::new();
    let bytes = client
        .get(&installer_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to download Forge installer")?
        .bytes()
        .await
        .context("Failed to read Forge installer")?;

    std::fs::write(&installer_path, &bytes)?;

    // Run installer headlessly
    let output = tokio::process::Command::new(java_path)
        .args([
            "-jar",
            &installer_path.to_string_lossy(),
            "--installClient",
            "--installDir",
            &versions_dir.to_string_lossy(),
        ])
        .output()
        .await
        .context("Failed to run Forge installer")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Forge installer failed: {}", stderr);
    }

    // Clean up installer
    let _ = tokio::fs::remove_file(&installer_path).await;

    // Forge installs to versions/forge-{gv}-{fv}/
    // The JSON might be at a slightly different path depending on version
    if profile_json.exists() {
        Ok(profile_json)
    } else {
        // Some Forge versions use a different naming
        let alt_dir = versions_dir.join(format!("{}-forge-{}", game_version, forge_version));
        let alt_json = alt_dir.join(format!("{}-forge-{}.json", game_version, forge_version));
        if alt_json.exists() {
            Ok(alt_json)
        } else {
            anyhow::bail!(
                "Forge installed but could not find profile JSON. Expected: {}",
                profile_json.display()
            )
        }
    }
}

// ── NeoForge ──────────────────────────────────────────────────

/// Fetch available NeoForge versions from Maven metadata.
pub async fn fetch_neoforge_versions(game_version: &str) -> Result<Vec<LoaderVersion>> {
    let client = reqwest::Client::new();

    // NeoForge uses semantic versioning: 21.0.1-beta for MC 1.21
    // The major version aligns with MC version
    let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

    let xml_text = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch NeoForge maven metadata")?
        .text()
        .await
        .context("Failed to read NeoForge maven metadata")?;

    // Parse XML to extract version list
    let mc_major: u32 = game_version
        .split('.')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut versions = Vec::new();
    // Simple XML parsing: find <version> tags
    for line in xml_text.lines() {
        let line = line.trim();
        if let Some(start) = line.find("<version>") {
            let rest = &line[start + 9..];
            if let Some(end) = rest.find("</version>") {
                let version = &rest[..end];
                // NeoForge version starts with MC minor version
                if version.starts_with(&format!("{}.", mc_major)) {
                    let stable = !version.contains("beta")
                        && !version.contains("alpha")
                        && !version.contains("rc");
                    versions.push(LoaderVersion {
                        loader: ModLoader::NeoForge,
                        version: version.to_string(),
                        stable,
                        game_versions: vec![game_version.to_string()],
                    });
                }
            }
        }
    }

    Ok(versions)
}

/// Install NeoForge.
pub async fn install_neoforge(
    game_version: &str,
    neoforge_version: &str,
    versions_dir: &Path,
    java_path: &Path,
) -> Result<PathBuf> {
    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
        neoforge_version, neoforge_version
    );

    let profile_id = format!("neoforge-{}", neoforge_version);
    let version_dir = versions_dir.join(&profile_id);
    let profile_json = version_dir.join(format!("{}.json", profile_id));

    if profile_json.exists() {
        return Ok(profile_json);
    }

    // Download and run installer (similar to Forge)
    let temp_dir = std::env::temp_dir().join("omnilauncher-neoforge");
    std::fs::create_dir_all(&temp_dir)?;
    let installer_path = temp_dir.join(format!("neoforge-{}-installer.jar", neoforge_version));

    let client = reqwest::Client::new();
    let bytes = client
        .get(&installer_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to download NeoForge installer")?
        .bytes()
        .await
        .context("Failed to read NeoForge installer")?;

    std::fs::write(&installer_path, &bytes)?;

    let output = tokio::process::Command::new(java_path)
        .args([
            "-jar",
            &installer_path.to_string_lossy(),
            "--installClient",
            "--installDir",
            &versions_dir.to_string_lossy(),
        ])
        .output()
        .await
        .context("Failed to run NeoForge installer")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("NeoForge installer failed: {}", stderr);
    }

    let _ = tokio::fs::remove_file(&installer_path).await;

    if profile_json.exists() {
        Ok(profile_json)
    } else {
        anyhow::bail!(
            "NeoForge installed but profile JSON not found at {}",
            profile_json.display()
        )
    }
}

// ── Quilt ─────────────────────────────────────────────────────

#[derive(Deserialize)]
struct QuiltLoaderEntry {
    version: String,
    separator: Option<String>,
    build: u32,
    maven: String,
    stable: bool,
}

/// Fetch available Quilt loader versions.
pub async fn fetch_quilt_loaders(game_version: &str) -> Result<Vec<LoaderVersion>> {
    let client = reqwest::Client::new();

    let loaders: Vec<QuiltLoaderEntry> = client
        .get("https://meta.quiltmc.org/v3/versions/loader")
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch Quilt loader versions")?
        .json()
        .await
        .context("Failed to parse Quilt loader versions")?;

    Ok(loaders
        .into_iter()
        .map(|l| LoaderVersion {
            loader: ModLoader::Quilt,
            version: l.version,
            stable: l.stable,
            game_versions: vec![game_version.to_string()],
        })
        .collect())
}

/// Fetch Quilt version profile.
pub async fn fetch_quilt_profile(game_version: &str, loader_version: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://meta.quiltmc.org/v3/versions/loader/{}/{}/profile/json",
        game_version, loader_version
    );

    let profile: serde_json::Value = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch Quilt profile")?
        .json()
        .await
        .context("Failed to parse Quilt profile")?;

    Ok(profile)
}

/// Install Quilt for an instance.
pub async fn install_quilt(
    game_version: &str,
    loader_version: &str,
    versions_dir: &Path,
) -> Result<PathBuf> {
    let profile = fetch_quilt_profile(game_version, loader_version).await?;

    let profile_id = format!("quilt-loader-{}-{}", loader_version, game_version);
    let version_dir = versions_dir.join(&profile_id);
    std::fs::create_dir_all(&version_dir)?;

    let profile_path = version_dir.join(format!("{}.json", profile_id));
    let json = serde_json::to_string_pretty(&profile)?;
    std::fs::write(&profile_path, json)?;

    log::info!("Installed Quilt {} for MC {}", loader_version, game_version);
    Ok(profile_path)
}

// ── Unified install interface ─────────────────────────────────

/// Install any mod loader for an instance.
pub async fn install_loader(
    loader: &ModLoader,
    game_version: &str,
    loader_version: &str,
    versions_dir: &Path,
    java_path: Option<&Path>,
) -> Result<PathBuf> {
    match loader {
        ModLoader::Vanilla => {
            // No loader needed, return vanilla version path
            let path = versions_dir
                .join(game_version)
                .join(format!("{}.json", game_version));
            if path.exists() {
                Ok(path)
            } else {
                anyhow::bail!("Vanilla version {} not yet downloaded", game_version)
            }
        }
        ModLoader::Fabric => install_fabric(game_version, loader_version, versions_dir).await,
        ModLoader::Forge => {
            let java = java_path.ok_or_else(|| anyhow::anyhow!("Java path required for Forge installation"))?;
            install_forge(game_version, loader_version, versions_dir, java).await
        }
        ModLoader::NeoForge => {
            let java = java_path.ok_or_else(|| anyhow::anyhow!("Java path required for NeoForge installation"))?;
            install_neoforge(game_version, loader_version, versions_dir, java).await
        }
        ModLoader::Quilt => install_quilt(game_version, loader_version, versions_dir).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_loader_from_str() {
        assert_eq!(ModLoader::from_str("fabric"), ModLoader::Fabric);
        assert_eq!(ModLoader::from_str("FORGE"), ModLoader::Forge);
        assert_eq!(ModLoader::from_str("NeoForge"), ModLoader::NeoForge);
        assert_eq!(ModLoader::from_str("quilt"), ModLoader::Quilt);
        assert_eq!(ModLoader::from_str("unknown"), ModLoader::Vanilla);
    }

    #[test]
    fn test_forge_installer_url() {
        let url = forge_installer_url("1.20.1", "47.2.0");
        assert!(url.contains("forge-1.20.1-47.2.0"));
        assert!(url.contains("installer.jar"));
    }
}
