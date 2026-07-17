use serde::{Deserialize, Serialize};
use tauri::command;

/// Minecraft version manifest entry
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

/// Full version manifest from Mojang
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

/// Version details (from a specific version JSON)
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

/// Fetches the Minecraft version manifest from Mojang.
#[command]
pub async fn get_version_manifest() -> Result<VersionManifest, String> {
    // TODO: GET https://piston-meta.mojang.com/mc/game/version_manifest_v2.json
    Err("Not yet implemented".to_string())
}

/// Fetches details for a specific Minecraft version.
#[command]
pub async fn get_version_details(version_id: String) -> Result<VersionDetails, String> {
    // TODO: GET the version JSON URL from manifest, parse and return
    Err("Not yet implemented".to_string())
}

/// Downloads all files needed for a Minecraft version into an instance.
/// This includes: version JSON, client JAR, libraries, asset index, assets.
#[command]
pub async fn download_version(instance_id: String, version_id: String) -> Result<(), String> {
    // TODO:
    // 1. Fetch version manifest, find version URL
    // 2. Download and parse version JSON
    // 3. Download client JAR to .minecraft/versions/<id>/<id>.jar
    // 4. Download libraries to .minecraft/libraries/ (respecting rules/natives)
    // 5. Download asset index to .minecraft/assets/indexes/<id>.json
    // 6. Download all assets to .minecraft/assets/objects/<prefix>/<hash>
    // 7. Update instance status in DB
    // 8. Emit progress events to frontend
    Err("Not yet implemented".to_string())
}
