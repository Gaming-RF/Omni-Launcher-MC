// Modrinth adapter — converts Modrinth API responses to unified types

use super::*;
use crate::api::modrinth;

const BASE_URL: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = "OmniLauncherMC/0.2.0 (github.com/Gaming-RF/Omni-Launcher-MC)";

fn map_project_type(pt: &str) -> String {
    match pt {
        "mod" => "mod".to_string(),
        "modpack" => "modpack".to_string(),
        "resourcepack" => "resourcepack".to_string(),
        "shader" => "shader".to_string(),
        "datapack" => "datapack".to_string(),
        other => other.to_string(),
    }
}

/// Search Modrinth and return unified results
pub async fn search(req: &UnifiedSearchRequest) -> anyhow::Result<Vec<UnifiedSearchResult>> {
    let mut url = format!(
        "{}/search?query={}&limit={}",
        BASE_URL,
        urlencoding::encode(&req.query),
        req.limit.unwrap_or(20)
    );

    if let Some(offset) = req.offset {
        url.push_str(&format!("&offset={offset}"));
    }

    if let Some(ref gv) = req.game_version {
        url.push_str(&format!("&filters=versions:{gv}"));
    }

    // Map sort order
    let index = match req.sort.as_ref().unwrap_or(&SortOrder::Relevance) {
        SortOrder::Relevance => "relevance",
        SortOrder::Downloads => "downloads",
        SortOrder::Updated => "updated",
        SortOrder::Newest => "newest",
        SortOrder::Follows => "follows",
    };
    url.push_str(&format!("&index={index}"));

    // Map project type filter
    if let Some(ref rt) = req.resource_type {
        let facet = match rt {
            ResourceType::Mod => "project_type:mod",
            ResourceType::Modpack => "project_type:modpack",
            ResourceType::ResourcePack => "project_type:resourcepack",
            ResourceType::Shader => "project_type:shader",
            ResourceType::DataPack => "project_type:datapack",
        };
        url.push_str(&format!("&facets=[[\"{facet}\"]]"));
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json::<modrinth::SearchResults>()
        .await?;

    Ok(resp
        .hits
        .into_iter()
        .map(|h| UnifiedSearchResult {
            source: ModSource::Modrinth,
            project_id: h.project_id,
            slug: h.slug,
            title: h.title,
            description: h.description,
            author: h.author,
            icon_url: h.icon_url,
            downloads: h.downloads,
            categories: h.display_categories.unwrap_or_default(),
            project_type: map_project_type(&h.project_type),
        })
        .collect())
}

/// Get versions for a Modrinth project
pub async fn get_versions(
    project_id: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
) -> anyhow::Result<Vec<UnifiedModVersion>> {
    let mut url = format!("{BASE_URL}/project/{project_id}/version");
    let mut params = vec![];
    if let Some(gv) = game_version {
        params.push(format!("game_versions=[\"{gv}\"]"));
    }
    if let Some(l) = loader {
        params.push(format!("loaders=[\"{l}\"]"));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<modrinth::ProjectVersion>>()
        .await?;

    Ok(resp
        .into_iter()
        .map(|v| UnifiedModVersion {
            source: ModSource::Modrinth,
            version_id: v.id,
            name: v.name,
            version_number: v.version_number,
            game_versions: v.game_versions,
            loaders: v.loaders,
            downloads: v.downloads,
            date_published: v.date_published,
            files: v
                .files
                .into_iter()
                .map(|f| UnifiedFile {
                    filename: f.filename,
                    url: f.url,
                    size: f.size,
                    primary: f.primary,
                    sha1: Some(f.hashes.sha1),
                    sha512: Some(f.hashes.sha512),
                })
                .collect(),
            dependencies: v
                .dependencies
                .into_iter()
                .map(|d| UnifiedDependency {
                    project_id: d.project_id.unwrap_or_default(),
                    version_id: d.version_id,
                    dependency_type: d.dependency_type,
                })
                .collect(),
            changelog: v.changelog,
        })
        .collect())
}

/// Get project details from Modrinth
pub async fn get_project(project_id: &str) -> anyhow::Result<UnifiedProjectDetails> {
    let url = format!("{BASE_URL}/project/{project_id}");
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    Ok(UnifiedProjectDetails {
        source: ModSource::Modrinth,
        project_id: resp["id"].as_str().unwrap_or_default().to_string(),
        slug: resp["slug"].as_str().unwrap_or_default().to_string(),
        title: resp["title"].as_str().unwrap_or_default().to_string(),
        description: resp["description"].as_str().unwrap_or_default().to_string(),
        body: resp["body"].as_str().map(String::from),
        author: resp["author"].as_str().unwrap_or("Unknown").to_string(),
        icon_url: resp["icon_url"].as_str().unwrap_or_default().to_string(),
        downloads: resp["downloads"].as_u64().unwrap_or(0),
        categories: resp["categories"]
            .as_array()
            .map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        project_type: map_project_type(resp["project_type"].as_str().unwrap_or("mod")),
        source_url: resp["source_url"].as_str().map(String::from),
        wiki_url: resp["wiki_url"].as_str().map(String::from),
        issues_url: resp["issues_url"].as_str().map(String::from),
        date_created: resp["date_created"].as_str().unwrap_or_default().to_string(),
        date_modified: resp["date_modified"].as_str().unwrap_or_default().to_string(),
    })
}
