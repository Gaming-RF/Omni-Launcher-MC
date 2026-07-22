// CurseForge adapter — converts CurseForge API responses to unified types

use super::*;
use crate::api::curseforge;
use std::env;

fn cf_api_key() -> String {
    env::var("CURSEFORGE_API_KEY").unwrap_or_default()
}

/// Search CurseForge and return unified results
pub async fn search(req: &UnifiedSearchRequest) -> anyhow::Result<Vec<UnifiedSearchResult>> {
    let api_key = cf_api_key();
    if api_key.is_empty() {
        // No API key configured — return empty
        return Ok(vec![]);
    }

    let class_id = req.resource_type.as_ref().map(|rt| match rt {
        ResourceType::Mod => 6,           // Mods
        ResourceType::Modpack => 4471,    // Modpacks
        ResourceType::ResourcePack => 12, // Texture Packs
        ResourceType::Shader => 6552,     // Shaders
        ResourceType::DataPack => 6945,   // Data Packs
    });

    let sort_field = match req.sort.as_ref().unwrap_or(&SortOrder::Relevance) {
        SortOrder::Downloads => 2,
        SortOrder::Updated => 3,
        SortOrder::Newest => 1,
        _ => 1, // CurseForge doesn't have relevance sort; default to popularity
    };

    let limit = req.limit.unwrap_or(20);
    let offset = req.offset.unwrap_or(0) as i32;

    let client = reqwest::Client::new();
    let mut body = serde_json::json!({
        "gameId": 432,
        "searchFilter": req.query,
        "sortField": sort_field,
        "sortOrder": "desc",
        "pageSize": limit,
        "index": offset,
    });

    if let Some(gv) = &req.game_version {
        body["gameVersion"] = serde_json::json!(gv);
    }

    if let Some(cid) = class_id {
        body["classId"] = serde_json::json!(cid);
    }

    let resp = client
        .post(format!("{}/v1/mods/search", curseforge::BASE_URL))
        .header("x-api-key", &api_key)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<curseforge::SearchResponse>()
        .await?;

    Ok(resp
        .data
        .into_iter()
        .map(|m| {
            let categories: Vec<String> = m
                .categories
                .unwrap_or_default()
                .into_iter()
                .map(|c| c.name)
                .collect();

            let author = m
                .authors
                .unwrap_or_default()
                .first()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            UnifiedSearchResult {
                source: ModSource::CurseForge,
                project_id: m.id.to_string(),
                slug: m.slug,
                title: m.name,
                description: m.summary,
                author,
                icon_url: m.logo.and_then(|l| l.thumbnail_url).unwrap_or_default(),
                downloads: m.download_count.max(0) as u64,
                categories,
                project_type: "mod".to_string(), // CurseForge doesn't have per-hit type
            }
        })
        .collect())
}

/// Get versions for a CurseForge mod
pub async fn get_versions(
    mod_id: &str,
    game_version: Option<&str>,
    _loader: Option<&str>,
) -> anyhow::Result<Vec<UnifiedModVersion>> {
    let api_key = cf_api_key();
    if api_key.is_empty() {
        return Ok(vec![]);
    }

    let id: i32 = mod_id.parse()?;
    let mut url = format!("{}/v1/mods/{}/files?pageSize=50", curseforge::BASE_URL, id);
    if let Some(gv) = game_version {
        url.push_str(&format!("&gameVersion={gv}"));
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("x-api-key", &api_key)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    let files = resp["data"].as_array().cloned().unwrap_or_default();

    Ok(files
        .into_iter()
        .filter_map(|f| {
            let file_id = f["id"].as_i64()?;
            let filename = f["fileName"].as_str()?.to_string();
            let display_name = f["displayName"].as_str().unwrap_or(&filename).to_string();
            let dl_count = f["downloadCount"].as_i64().unwrap_or(0).max(0) as u64;
            let file_date = f["fileDate"].as_str().unwrap_or("").to_string();
            let file_len = f["fileLength"].as_i64().unwrap_or(0).max(0) as u64;

            let game_versions: Vec<String> = f["gameVersions"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let hashes = f["hashes"].as_array().cloned().unwrap_or_default();
            let sha1 = hashes.iter().find_map(|h| {
                if h["algo"].as_i64() == Some(1) {
                    h["value"].as_str().map(String::from)
                } else {
                    None
                }
            });

            let deps = f["dependencies"].as_array().cloned().unwrap_or_default();
            let dependencies = deps
                .into_iter()
                .filter_map(|d| {
                    Some(UnifiedDependency {
                        project_id: d["modId"].as_i64()?.to_string(),
                        version_id: d["fileId"].as_i64().map(|id| id.to_string()),
                        dependency_type: match d["relationType"].as_i64() {
                            Some(1) => "embedded",
                            Some(2) => "optional",
                            Some(3) => "required",
                            _ => "optional",
                        }
                        .to_string(),
                    })
                })
                .collect();

            Some(UnifiedModVersion {
                source: ModSource::CurseForge,
                version_id: file_id.to_string(),
                name: display_name,
                version_number: filename.clone(),
                game_versions,
                loaders: vec![], // CurseForge doesn't always expose loader in file data
                downloads: dl_count,
                date_published: file_date,
                files: vec![UnifiedFile {
                    filename,
                    url: String::new(), // Need separate API call for download URL
                    size: file_len,
                    primary: true,
                    sha1,
                    sha512: None,
                }],
                dependencies,
                changelog: None,
            })
        })
        .collect())
}

/// Get project details from CurseForge
pub async fn get_project(mod_id: &str) -> anyhow::Result<UnifiedProjectDetails> {
    let api_key = cf_api_key();
    if api_key.is_empty() {
        anyhow::bail!("CurseForge API key not configured");
    }

    let id: i32 = mod_id.parse()?;
    let url = format!("{}/v1/mods/{}", curseforge::BASE_URL, id);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("x-api-key", &api_key)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    let m = &resp["data"];
    let author = m["authors"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|a| a["name"].as_str())
        .unwrap_or("Unknown")
        .to_string();

    Ok(UnifiedProjectDetails {
        source: ModSource::CurseForge,
        project_id: m["id"].as_i64().unwrap_or(0).to_string(),
        slug: m["slug"].as_str().unwrap_or_default().to_string(),
        title: m["name"].as_str().unwrap_or_default().to_string(),
        description: m["summary"].as_str().unwrap_or_default().to_string(),
        body: m["description"].as_str().map(String::from),
        author,
        icon_url: m["logo"]["thumbnailUrl"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        downloads: m["downloadCount"].as_i64().unwrap_or(0).max(0) as u64,
        categories: m["categories"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|c| c["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        project_type: "mod".to_string(),
        source_url: m["links"]["sourceUrl"].as_str().map(String::from),
        wiki_url: m["links"]["wikiUrl"].as_str().map(String::from),
        issues_url: m["links"]["issuesUrl"].as_str().map(String::from),
        date_created: m["dateCreated"].as_str().unwrap_or_default().to_string(),
        date_modified: m["dateModified"].as_str().unwrap_or_default().to_string(),
    })
}
