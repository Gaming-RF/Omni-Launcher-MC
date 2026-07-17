// NeoForge mod loader installer
// Maven: https://maven.neoforged.net/releases/net/neoforged/neoforge/
//
// NeoForge is a fork of Forge (post-1.20.1). The installation process is
// identical to Forge — download installer JAR, extract version JSON.
// Version scheme changed: NeoForge uses "24.1.1" style instead of "49.0.1".

use anyhow::{Context, Result};
use std::io::Read;
use std::path::Path;

const MAVEN_BASE: &str = "https://maven.neoforged.net/releases";
const USER_AGENT: &str = "OmniLauncherMC/0.1.0";

/// Get the download URL for a NeoForge installer JAR.
fn installer_url(_mc_version: &str, neoforge_version: &str) -> String {
    format!(
        "{}/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
        MAVEN_BASE, neoforge_version, neoforge_version
    )
}

/// Install NeoForge for an instance.
pub async fn install(
    base_dir: &Path,
    mc_version: &str,
    neoforge_version: &str,
) -> Result<String> {
    let profile_id = format!("neoforge-{}-{}", mc_version, neoforge_version);
    let version_dir = base_dir.join("versions").join(&profile_id);
    std::fs::create_dir_all(&version_dir)?;

    let url = installer_url(mc_version, neoforge_version);

    let client = reqwest::Client::new();
    let bytes = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to download NeoForge installer")?
        .bytes()
        .await
        .context("Failed to read NeoForge installer response")?;

    // Write installer to temp file
    let installer_path = base_dir
        .join("cache")
        .join(format!("neoforge-{}-installer.jar", profile_id));
    if let Some(parent) = installer_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&installer_path, &bytes)?;

    // Extract version JSON (same process as Forge)
    let version_json = extract_version_json(&installer_path, mc_version, neoforge_version)?;

    let json_path = version_dir.join(format!("{}.json", profile_id));
    std::fs::write(&json_path, serde_json::to_string_pretty(&version_json)?)?;

    let _ = std::fs::remove_file(&installer_path);

    log::info!(
        "Installed NeoForge {} for MC {}",
        neoforge_version,
        mc_version
    );
    Ok(profile_id)
}

fn extract_version_json(
    installer_path: &Path,
    mc_version: &str,
    neoforge_version: &str,
) -> Result<serde_json::Value> {
    let file = std::fs::File::open(installer_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    if let Ok(mut f) = archive.by_name("install_profile.json") {
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let profile: serde_json::Value = serde_json::from_str(&contents)?;

        if let Some(version_info) = profile.get("versionInfo") {
            let mut vi = version_info.clone();
            vi["id"] = serde_json::Value::String(format!(
                "neoforge-{}-{}",
                mc_version, neoforge_version
            ));
            return Ok(vi);
        }
        return Ok(profile);
    }

    if let Ok(mut f) = archive.by_name("version.json") {
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let mut json: serde_json::Value = serde_json::from_str(&contents)?;
        json["id"] = serde_json::Value::String(format!(
            "neoforge-{}-{}",
            mc_version, neoforge_version
        ));
        return Ok(json);
    }

    anyhow::bail!(
        "Could not find install_profile.json or version.json in NeoForge installer"
    )
}

/// Get available NeoForge versions for a Minecraft version.
/// NeoForge versions start with the MC version number (e.g., "24.1.1" for MC 1.21.x).
pub async fn get_neoforge_versions(mc_version: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/net/neoforged/neoforge/maven-metadata.xml",
        MAVEN_BASE
    );

    let resp = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to fetch NeoForge maven metadata")?
        .text()
        .await?;

    // NeoForge version scheme: MC 1.20.x -> NeoForge 20.x.y, MC 1.21.x -> 21.x.y
    // The major version corresponds to MC minor version
    let mc_minor: u32 = mc_version
        .split('.')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let prefix = format!("{}.", mc_minor + 1); // NeoForge = MC minor + 1

    let versions: Vec<String> = resp
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
                let v = &trimmed[9..trimmed.len() - 10];
                if v.starts_with(&prefix) {
                    return Some(v.to_string());
                }
            }
            None
        })
        .collect();

    Ok(versions)
}
