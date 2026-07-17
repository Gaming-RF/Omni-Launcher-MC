// Fabric mod loader installer
// Meta API: https://meta.fabricmc.net/v2
//
// Process:
// 1. Fetch available loader versions for a Minecraft version
// 2. Download the version manifest (a merged vanilla+fabric JSON)
// 3. Write it to versions/<id>/<id>.json
// 4. The launcher's existing prepare() handles libraries/assets from there

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const META_BASE: &str = "https://meta.fabricmc.net/v2";
const USER_AGENT: &str = "OmniLauncherMC/0.1.0";

#[derive(Debug, Serialize, Deserialize)]
pub struct FabricLoaderVersion {
    pub separator: String,
    pub build: u32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

/// Each entry from the Fabric meta API wraps loader + intermediary info.
#[derive(Debug, Deserialize)]
struct FabricVersionEntry {
    loader: FabricLoaderVersion,
    // intermediary and launcherMeta are present but not needed for listing
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FabricGameVersion {
    pub version: String,
    pub stable: bool,
}

/// Get available Fabric loader versions for a given Minecraft version.
pub async fn get_loader_versions(mc_version: &str) -> Result<Vec<FabricLoaderVersion>> {
    let client = reqwest::Client::new();
    let url = format!("{}/versions/loader/{}", META_BASE, mc_version);

    let entries: Vec<FabricVersionEntry> = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch Fabric loader versions")?
        .json()
        .await
        .context("Failed to parse Fabric loader versions")?;

    Ok(entries.into_iter().map(|e| e.loader).collect())
}

/// Get the latest stable Fabric loader version for a Minecraft version.
pub async fn get_latest_loader(mc_version: &str) -> Result<FabricLoaderVersion> {
    let versions = get_loader_versions(mc_version).await?;
    versions
        .into_iter()
        .find(|v| v.stable)
        .context("No stable Fabric loader found for this MC version")
}

/// Get the Fabric version JSON (merged vanilla + loader profile).
/// This is the JSON that goes into versions/<profile_id>/<profile_id>.json
pub async fn get_version_json(mc_version: &str, loader_version: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/versions/loader/{}/{}/profile/json",
        META_BASE, mc_version, loader_version
    );

    let json: serde_json::Value = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch Fabric version JSON")?
        .json()
        .await
        .context("Failed to parse Fabric version JSON")?;

    Ok(json)
}

/// Install Fabric for an instance.
/// Downloads the merged version JSON and returns the profile ID.
pub async fn install(
    base_dir: &std::path::Path,
    mc_version: &str,
    loader_version: &str,
) -> Result<String> {
    let profile_id = format!("fabric-loader-{}-{}", loader_version, mc_version);
    let version_dir = base_dir.join("versions").join(&profile_id);
    std::fs::create_dir_all(&version_dir)?;

    let json = get_version_json(mc_version, loader_version).await?;
    let json_path = version_dir.join(format!("{}.json", profile_id));
    std::fs::write(&json_path, serde_json::to_string_pretty(&json)?)?;

    log::info!("Installed Fabric {} for MC {}", loader_version, mc_version);
    Ok(profile_id)
}
