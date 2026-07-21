// Unified mod platform abstraction
// Provides a common interface for Modrinth and CurseForge APIs

pub mod modrinth_adapter;
pub mod curseforge_adapter;

use serde::{Deserialize, Serialize};

/// Which platform a mod comes from
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModSource {
    Modrinth,
    CurseForge,
}

impl ModSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModSource::Modrinth => "modrinth",
            ModSource::CurseForge => "curseforge",
        }
    }
}

/// Unified search result from any platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchResult {
    pub source: ModSource,
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub icon_url: String,
    pub downloads: u64,
    pub categories: Vec<String>,
    pub project_type: String, // "mod", "modpack", "resourcepack", "shader"
}

/// Unified version info from any platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedModVersion {
    pub source: ModSource,
    pub version_id: String,
    pub name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub downloads: u64,
    pub date_published: String,
    pub files: Vec<UnifiedFile>,
    pub dependencies: Vec<UnifiedDependency>,
    pub changelog: Option<String>,
}

/// A downloadable file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedFile {
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub primary: bool,
    pub sha1: Option<String>,
    pub sha512: Option<String>,
}

/// A mod dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDependency {
    pub project_id: String,
    pub version_id: Option<String>,
    pub dependency_type: String, // "required", "optional", "incompatible", "embedded"
}

/// Unified project details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedProjectDetails {
    pub source: ModSource,
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: Option<String>,
    pub author: String,
    pub icon_url: String,
    pub downloads: u64,
    pub categories: Vec<String>,
    pub project_type: String,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub issues_url: Option<String>,
    pub date_created: String,
    pub date_modified: String,
}

/// Resource type filter for search
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Mod,
    Modpack,
    ResourcePack,
    Shader,
    DataPack,
}

impl ResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Mod => "mod",
            ResourceType::Modpack => "modpack",
            ResourceType::ResourcePack => "resourcepack",
            ResourceType::Shader => "shader",
            ResourceType::DataPack => "datapack",
        }
    }
}

/// Sort options for search
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Relevance,
    Downloads,
    Updated,
    Newest,
    Follows,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Relevance => "relevance",
            SortOrder::Downloads => "downloads",
            SortOrder::Updated => "updated",
            SortOrder::Newest => "newest",
            SortOrder::Follows => "follows",
        }
    }
}

/// Unified search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchRequest {
    pub query: String,
    pub source: Option<ModSource>,       // None = search both
    pub resource_type: Option<ResourceType>, // None = all types
    pub game_version: Option<String>,
    pub loader: Option<String>,
    pub sort: Option<SortOrder>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

/// Unified search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchResponse {
    pub results: Vec<UnifiedSearchResult>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
    pub source_counts: SourceCounts,
}

/// How many results from each source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCounts {
    pub modrinth: u32,
    pub curseforge: u32,
}

/// Search both platforms and merge results
pub async fn search_unified(req: &UnifiedSearchRequest) -> anyhow::Result<UnifiedSearchResponse> {
    use futures::future::join;

    let modrinth_future = async {
        if req.source.as_ref() == Some(&ModSource::CurseForge) {
            return Ok::<Vec<UnifiedSearchResult>, anyhow::Error>(vec![]);
        }
        modrinth_adapter::search(req).await
    };

    let curseforge_future = async {
        if req.source.as_ref() == Some(&ModSource::Modrinth) {
            return Ok::<Vec<UnifiedSearchResult>, anyhow::Error>(vec![]);
        }
        curseforge_adapter::search(req).await
    };

    let (modrinth_result, curseforge_result) = join(modrinth_future, curseforge_future).await;

    let mut results = Vec::new();
    let mut modrinth_count = 0u32;
    let mut curseforge_count = 0u32;

    match modrinth_result {
        Ok(hits) => {
            modrinth_count = hits.len() as u32;
            results.extend(hits);
        }
        Err(e) => eprintln!("[platform] Modrinth search failed: {e}"),
    }

    match curseforge_result {
        Ok(hits) => {
            curseforge_count = hits.len() as u32;
            results.extend(hits);
        }
        Err(e) => eprintln!("[platform] CurseForge search failed: {e}"),
    }

    // Sort merged results by downloads (descending)
    results.sort_by(|a, b| b.downloads.cmp(&a.downloads));

    // Apply pagination
    let total = results.len() as u32;
    let offset = req.offset.unwrap_or(0);
    let limit = req.limit.unwrap_or(20);
    let start = offset as usize;
    let end = (start + limit as usize).min(results.len());
    let page = if start < results.len() {
        results[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(UnifiedSearchResponse {
        results: page,
        total,
        offset,
        limit,
        source_counts: SourceCounts {
            modrinth: modrinth_count,
            curseforge: curseforge_count,
        },
    })
}

/// Get versions for a project from its source
pub async fn get_project_versions(
    source: &ModSource,
    project_id: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
) -> anyhow::Result<Vec<UnifiedModVersion>> {
    match source {
        ModSource::Modrinth => {
            modrinth_adapter::get_versions(project_id, game_version, loader).await
        }
        ModSource::CurseForge => {
            curseforge_adapter::get_versions(project_id, game_version, loader).await
        }
    }
}

/// Get project details from its source
pub async fn get_project_details(
    source: &ModSource,
    project_id: &str,
) -> anyhow::Result<UnifiedProjectDetails> {
    match source {
        ModSource::Modrinth => modrinth_adapter::get_project(project_id).await,
        ModSource::CurseForge => curseforge_adapter::get_project(project_id).await,
    }
}
