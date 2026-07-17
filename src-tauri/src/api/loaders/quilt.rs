// Quilt mod loader installer
// Meta API: https://meta.quiltmc.org/v3
//
// Very similar process to Fabric — Quilt forked from Fabric and uses a
// compatible meta API structure.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const META_BASE: &str = "https://meta.quiltmc.org/v3";
const USER_AGENT: &str = "OmniLauncherMC/0.1.0";

#[derive(Debug, Serialize, Deserialize)]
pub struct QuiltLoaderVersion {
    pub separator: String,
    pub build: u32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

/// Get available Quilt loader versions for a given Minecraft version.
pub async fn get_loader_versions(mc_version: &str) -> Result<Vec<QuiltLoaderVersion>> {
    let client = reqwest::Client::new();
    let url = format!("{}/versions/loader/{}", META_BASE, mc_version);

    let versions: Vec<QuiltLoaderVersion> = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch Quilt loader versions")?
        .json()
        .await
        .context("Failed to parse Quilt loader versions")?;

    Ok(versions)
}

/// Get the latest stable Quilt loader version.
pub async fn get_latest_loader(mc_version: &str) -> Result<QuiltLoaderVersion> {
    let versions = get_loader_versions(mc_version).await?;
    versions
        .into_iter()
        .find(|v| v.stable)
        .context("No stable Quilt loader found for this MC version")
}

/// Get the Quilt version JSON.
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
        .context("Failed to fetch Quilt version JSON")?
        .json()
        .await
        .context("Failed to parse Quilt version JSON")?;

    Ok(json)
}

/// Install Quilt for an instance.
pub async fn install(
    base_dir: &std::path::Path,
    mc_version: &str,
    loader_version: &str,
) -> Result<String> {
    let profile_id = format!("quilt-loader-{}-{}", loader_version, mc_version);
    let version_dir = base_dir.join("versions").join(&profile_id);
    std::fs::create_dir_all(&version_dir)?;

    let json = get_version_json(mc_version, loader_version).await?;
    let json_path = version_dir.join(format!("{}.json", profile_id));
    std::fs::write(&json_path, serde_json::to_string_pretty(&json)?)?;

    log::info!("Installed Quilt {} for MC {}", loader_version, mc_version);
    Ok(profile_id)
}
