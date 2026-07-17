// Forge mod loader installer
// Maven: https://maven.minecraftforge.net/net/minecraftforge/forge/
//
// Forge installation is more complex than Fabric/Quilt:
// 1. Download the Forge installer JAR
// 2. Extract install_profile.json from the JAR
// 3. The profile contains a "versionInfo" object that IS the version JSON
// 4. Write it to versions/<id>/<id>.json
// 5. Forge libraries are downloaded from maven.minecraftforge.net
//
// For newer Forge (1.13+), the installer also has a "processors" step that
// patches the game JAR. We skip that for now and use the profile JSON approach
// which works for most cases.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;

const MAVEN_BASE: &str = "https://maven.minecraftforge.net";
const USER_AGENT: &str = "OmniLauncherMC/0.1.0";

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeVersion {
    pub mc_version: String,
    pub forge_version: String,
    pub build: u32,
}

/// Get the download URL for a Forge installer JAR.
fn installer_url(mc_version: &str, forge_version: &str) -> String {
    format!(
        "{}/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
        MAVEN_BASE, mc_version, forge_version, mc_version, forge_version
    )
}

/// Install Forge for an instance.
/// Downloads the installer, extracts the version JSON, and writes it.
pub async fn install(
    base_dir: &Path,
    mc_version: &str,
    forge_version: &str,
) -> Result<String> {
    let profile_id = format!("forge-{}-{}", mc_version, forge_version);
    let version_dir = base_dir.join("versions").join(&profile_id);
    std::fs::create_dir_all(&version_dir)?;

    let url = installer_url(mc_version, forge_version);

    // Download installer JAR to a temp file
    let client = reqwest::Client::new();
    let bytes = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to download Forge installer")?
        .bytes()
        .await
        .context("Failed to read Forge installer response")?;

    // Write installer to temp file
    let installer_path = base_dir.join("cache").join(format!("forge-{}-installer.jar", profile_id));
    if let Some(parent) = installer_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&installer_path, &bytes)?;

    // Extract install_profile.json from the JAR
    let version_json = extract_version_json(&installer_path, mc_version, forge_version)?;

    let json_path = version_dir.join(format!("{}.json", profile_id));
    std::fs::write(&json_path, serde_json::to_string_pretty(&version_json)?)?;

    // Clean up installer
    let _ = std::fs::remove_file(&installer_path);

    log::info!("Installed Forge {} for MC {}", forge_version, mc_version);
    Ok(profile_id)
}

/// Extract the version JSON from a Forge installer JAR.
fn extract_version_json(
    installer_path: &Path,
    mc_version: &str,
    forge_version: &str,
) -> Result<serde_json::Value> {
    let file = std::fs::File::open(installer_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Try install_profile.json first (newer Forge)
    if let Ok(mut f) = archive.by_name("install_profile.json") {
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let profile: serde_json::Value = serde_json::from_str(&contents)?;

        // Newer Forge (1.13+) has "versionInfo" in install_profile.json
        if let Some(version_info) = profile.get("versionInfo") {
            let mut vi = version_info.clone();
            // Override the ID so it doesn't conflict
            vi["id"] = serde_json::Value::String(format!(
                "forge-{}-{}",
                mc_version, forge_version
            ));
            return Ok(vi);
        }

        // Older Forge has the version JSON directly
        return Ok(profile);
    }

    // Try version.json (some Forge versions)
    if let Ok(mut f) = archive.by_name("version.json") {
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let mut json: serde_json::Value = serde_json::from_str(&contents)?;
        json["id"] =
            serde_json::Value::String(format!("forge-{}-{}", mc_version, forge_version));
        return Ok(json);
    }

    anyhow::bail!(
        "Could not find install_profile.json or version.json in Forge installer"
    )
}

/// Get the Forge version string for a given MC version from the Maven metadata.
/// This queries the Maven directory listing which is XML.
pub async fn get_forge_versions(mc_version: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/net/minecraftforge/forge/maven-metadata.xml",
        MAVEN_BASE
    );

    let resp = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch Forge maven metadata")?
        .text()
        .await?;

    // Parse XML to find versions matching the MC version
    // Maven metadata lists versions like "1.20.4-49.0.1" where the prefix is MC version
    let prefix = format!("{}-", mc_version);
    let versions: Vec<String> = resp
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
                let v = &trimmed[9..trimmed.len() - 10];
                if v.starts_with(&prefix) {
                    return Some(v[prefix.len()..].to_string());
                }
            }
            None
        })
        .collect();

    Ok(versions)
}
