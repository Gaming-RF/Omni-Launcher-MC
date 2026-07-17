// Modrinth API v2 client
// Base URL: https://api.modrinth.com/v2
// Rate limit: 300 req/min per IP
// User-Agent required: OmniLauncherMC/<version> (contact)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = "OmniLauncherMC/0.1.0 (github.com/OmniLauncherMC)";

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub hits: Vec<SearchHit>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub categories: Option<Vec<String>>,
    pub client_side: String,
    pub server_side: String,
    pub project_type: String,
    pub downloads: u64,
    pub icon_url: String,
    pub color: Option<u32>,
    pub project_id: String,
    pub author: String,
    pub display_categories: Option<Vec<String>>,
    pub versions: Vec<String>,
    pub follows: u64,
    pub date_created: String,
    pub date_modified: String,
    #[serde(rename = "latest_version")]
    pub latest_version: Option<String>,
    pub license: Option<String>,
    pub gallery: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectVersion {
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub changelog_url: Option<String>,
    pub date_published: String,
    pub downloads: u64,
    pub version_type: String,
    pub status: String,
    pub requested_status: Option<String>,
    pub files: Vec<VersionFile>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionFile {
    pub hashes: FileHashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
    pub file_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileHashes {
    pub sha512: String,
    pub sha1: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub dependency_type: String,
}

/// Search Modrinth for mods/modpacks.
pub async fn search(
    query: &str,
    facets: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<SearchResults> {
    let client = reqwest::Client::new();
    let mut url = format!(
        "{}/search?query={}&offset={}&limit={}",
        BASE_URL,
        urlencoding::encode(query),
        offset,
        limit
    );
    if let Some(f) = facets {
        url.push_str(&format!("&facets={}", urlencoding::encode(f)));
    }

    let results: SearchResults = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Modrinth search request failed")?
        .json()
        .await
        .context("Failed to parse Modrinth search response")?;

    Ok(results)
}

/// Get all versions for a project.
pub async fn get_project_versions(
    project_id: &str,
    loaders: Option<&str>,
    game_versions: Option<&str>,
) -> Result<Vec<ProjectVersion>> {
    let client = reqwest::Client::new();
    let mut url = format!("{}/project/{}/version", BASE_URL, project_id);
    let mut params = vec![];
    if let Some(l) = loaders {
        params.push(format!("loaders=[\"{}\"]", l));
    }
    if let Some(gv) = game_versions {
        params.push(format!("game_versions=[\"{}\"]", gv));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let versions: Vec<ProjectVersion> = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Modrinth version request failed")?
        .json()
        .await
        .context("Failed to parse Modrinth versions")?;

    Ok(versions)
}

/// Get a specific version by ID.
pub async fn get_version(version_id: &str) -> Result<ProjectVersion> {
    let client = reqwest::Client::new();
    let version: ProjectVersion = client
        .get(format!("{}/version/{}", BASE_URL, version_id))
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Modrinth version fetch failed")?
        .json()
        .await
        .context("Failed to parse Modrinth version")?;

    Ok(version)
}

/// Get a version by file hash (SHA-512).
pub async fn get_version_by_hash(hash: &str) -> Result<ProjectVersion> {
    let client = reqwest::Client::new();
    let version: ProjectVersion = client
        .get(format!(
            "{}/version_file/{}?algorithm=sha512",
            BASE_URL, hash
        ))
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Modrinth hash lookup failed")?
        .json()
        .await
        .context("Failed to parse Modrinth hash lookup")?;

    Ok(version)
}
