// CurseForge CFCore API client
// Base URL: https://api.curseforge.com
// Auth: x-api-key header
// Game ID for Minecraft: 432

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.curseforge.com";
const MINECRAFT_GAME_ID: i32 = 432;

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
    pub relation_type: i32,
}

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
    #[derive(Deserialize)]
    struct Response {
        data: Mod,
    }

    let resp: Response = client
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
) -> Result<SearchResponse> {
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

    let resp: SearchResponse = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .context("CurseForge file list request failed")?
        .json()
        .await
        .context("Failed to parse CurseForge file list")?;

    Ok(resp)
}

/// Get the download URL for a specific file.
/// Returns None if the mod author has disabled third-party distribution.
pub async fn get_file_download_url(
    api_key: &str,
    mod_id: i32,
    file_id: i32,
) -> Result<Option<String>> {
    let client = reqwest::Client::new();
    #[derive(Deserialize)]
    struct Response {
        data: String,
    }

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

    let body: Response = resp
        .json()
        .await
        .context("Failed to parse CurseForge download URL response")?;

    Ok(Some(body.data))
}
