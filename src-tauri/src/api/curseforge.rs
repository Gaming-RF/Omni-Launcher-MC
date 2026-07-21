// CurseForge CFCore API client
// Base URL: https://api.curseforge.com
// Auth: x-api-key header
// Game ID for Minecraft: 432

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const BASE_URL: &str = "https://api.curseforge.com";
pub const MINECRAFT_GAME_ID: i32 = 432;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub data: Vec<Mod>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub index: i32,
    #[serde(rename = "pageSize")]
    pub page_size: i32,
    #[serde(rename = "resultCount")]
    pub result_count: i32,
    #[serde(rename = "totalCount")]
    pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mod {
    pub id: i32,
    #[serde(rename = "gameId")]
    pub game_id: i32,
    pub name: String,
    pub slug: String,
    pub links: Option<ModLinks>,
    pub summary: String,
    #[serde(rename = "downloadCount")]
    pub download_count: i64,
    pub categories: Option<Vec<Category>>,
    pub authors: Option<Vec<ModAuthor>>,
    pub logo: Option<ModAsset>,
    #[serde(rename = "latestFiles")]
    pub latest_files: Option<Vec<File>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModLinks {
    #[serde(rename = "websiteUrl")]
    pub website_url: Option<String>,
    #[serde(rename = "wikiUrl")]
    pub wiki_url: Option<String>,
    #[serde(rename = "issuesUrl")]
    pub issues_url: Option<String>,
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub url: Option<String>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModAuthor {
    pub id: i32,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModAsset {
    pub id: i32,
    pub title: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: i32,
    #[serde(rename = "gameId")]
    pub game_id: i32,
    #[serde(rename = "modId")]
    pub mod_id: i32,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "releaseType")]
    pub release_type: i32,
    #[serde(rename = "fileDate")]
    pub file_date: String,
    #[serde(rename = "fileLength")]
    pub file_length: Option<i64>,
    #[serde(rename = "downloadCount")]
    pub download_count: i64,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Vec<String>,
    pub hashes: Option<Vec<FileHash>>,
    pub dependencies: Option<Vec<FileDependency>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileHash {
    pub value: String,
    pub algo: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDependency {
    #[serde(rename = "modId")]
    pub mod_id: i32,
    #[serde(rename = "relationType")]
    pub relation_type: i32, // 1=embedded, 2=optional, 3=required, 4=tool, 5=incompatible, 6=include
}

/// Response wrapper for single-data endpoints.
#[derive(Debug, Deserialize)]
struct DataResponse<T> {
    data: T,
}

/// Response for the fingerprint endpoint.
#[derive(Debug, Deserialize)]
pub struct FingerprintResponse {
    pub data: FingerprintData,
}

#[derive(Debug, Deserialize)]
pub struct FingerprintData {
    #[serde(rename = "isCacheBuilt")]
    pub is_cache_built: bool,
    #[serde(rename = "exactMatches")]
    pub exact_matches: Vec<FingerprintMatch>,
}

#[derive(Debug, Deserialize)]
pub struct FingerprintMatch {
    pub id: i32,
    pub file: File,
    #[serde(rename = "latestFiles")]
    pub latest_files: Option<Vec<File>>,
}

// ── Public API ──────────────────────────────────────────────────

/// Search CurseForge for Minecraft mods.
pub async fn search_mods(
    api_key: &str,
    query: &str,
    game_version: Option<&str>,
    mod_loader: Option<&str>,
    index: i32,
    page_size: i32,
) -> Result<SearchResponse> {
    let client = reqwest::Client::new();
    let mut url = format!(
        "{}/v1/mods/search?gameId={}&searchFilter={}&index={}&pageSize={}",
        BASE_URL,
        MINECRAFT_GAME_ID,
        urlencoding::encode(query),
        index,
        page_size
    );
    if let Some(gv) = game_version {
        url.push_str(&format!("&gameVersion={}", urlencoding::encode(gv)));
    }
    if let Some(ml) = mod_loader {
        url.push_str(&format!("&modLoaderType={}", ml));
    }

    let resp: SearchResponse = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .context("CurseForge search request failed")?
        .json()
        .await
        .context("Failed to parse CurseForge search response")?;

    Ok(resp)
}

/// Get a single mod by ID.
pub async fn get_mod(api_key: &str, mod_id: i32) -> Result<Mod> {
    let client = reqwest::Client::new();
    let resp: DataResponse<Mod> = client
        .get(format!("{}/v1/mods/{}", BASE_URL, mod_id))
        .header("x-api-key", api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .context("CurseForge mod fetch failed")?
        .json()
        .await
        .context("Failed to parse CurseForge mod response")?;

    Ok(resp.data)
}

/// Get files for a mod, optionally filtered by game version and loader.
pub async fn get_mod_files(
    api_key: &str,
    mod_id: i32,
    game_version: Option<&str>,
    mod_loader: Option<&str>,
    index: i32,
    page_size: i32,
) -> Result<Vec<File>> {
    let client = reqwest::Client::new();
    let mut url = format!(
        "{}/v1/mods/{}/files?index={}&pageSize={}",
        BASE_URL, mod_id, index, page_size
    );
    if let Some(gv) = game_version {
        url.push_str(&format!("&gameVersion={}", urlencoding::encode(gv)));
    }
    if let Some(ml) = mod_loader {
        url.push_str(&format!("&modLoaderType={}", ml));
    }

    let resp: DataResponse<Vec<File>> = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .context("CurseForge file list request failed")?
        .json()
        .await
        .context("Failed to parse CurseForge file list")?;

    Ok(resp.data)
}

/// Get a specific file for a mod.
pub async fn get_mod_file(api_key: &str, mod_id: i32, file_id: i32) -> Result<File> {
    let client = reqwest::Client::new();
    let resp: DataResponse<File> = client
        .get(format!("{}/v1/mods/{}/files/{}", BASE_URL, mod_id, file_id))
        .header("x-api-key", api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .context("CurseForge file fetch failed")?
        .json()
        .await
        .context("Failed to parse CurseForge file")?;

    Ok(resp.data)
}

/// Get the download URL for a specific file.
/// Returns None if the mod author has disabled third-party distribution.
pub async fn get_file_download_url(
    api_key: &str,
    mod_id: i32,
    file_id: i32,
) -> Result<Option<String>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/v1/mods/{}/files/{}/download-url",
            BASE_URL, mod_id, file_id
        ))
        .header("x-api-key", api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .context("CurseForge download URL request failed")?;

    if resp.status() == reqwest::StatusCode::FORBIDDEN
        || resp.status() == reqwest::StatusCode::NOT_FOUND
    {
        return Ok(None);
    }

    let body: DataResponse<String> = resp
        .json()
        .await
        .context("Failed to parse CurseForge download URL response")?;

    Ok(Some(body.data))
}

/// Look up mods by file fingerprints (hashes).
/// Used for cross-source matching: Modrinth SHA-1 → CurseForge match.
/// `fingerprints` is a list of MurmurHash2 values (unsigned 32-bit).
pub async fn get_fingerprints(api_key: &str, fingerprints: &[u64]) -> Result<FingerprintResponse> {
    let client = reqwest::Client::new();
    let resp: FingerprintResponse = client
        .post(format!("{}/v1/fingerprints", BASE_URL))
        .header("x-api-key", api_key)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "fingerprints": fingerprints }))
        .send()
        .await
        .context("CurseForge fingerprint lookup failed")?
        .json()
        .await
        .context("Failed to parse CurseForge fingerprint response")?;

    Ok(resp)
}
