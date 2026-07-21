// Unified platform commands — search, versions, details from both Modrinth + CurseForge

use crate::platforms::{
    self, ModSource, ResourceType, SortOrder, UnifiedSearchRequest,
};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformSearchArgs {
    pub query: String,
    pub source: Option<String>,        // "modrinth", "curseforge", or null (both)
    pub resource_type: Option<String>, // "mod", "modpack", "resourcepack", "shader", "datapack"
    pub game_version: Option<String>,
    pub loader: Option<String>,
    pub sort: Option<String>, // "relevance", "downloads", "updated", "newest", "follows"
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

/// Unified search across Modrinth and CurseForge
#[command]
pub async fn search_mods_unified(args: PlatformSearchArgs) -> Result<String, String> {
    let source = args.source.as_deref().map(|s| match s {
        "modrinth" => ModSource::Modrinth,
        "curseforge" => ModSource::CurseForge,
        _ => ModSource::Modrinth,
    });

    let resource_type = args.resource_type.as_deref().map(|r| match r {
        "mod" => ResourceType::Mod,
        "modpack" => ResourceType::Modpack,
        "resourcepack" => ResourceType::ResourcePack,
        "shader" => ResourceType::Shader,
        "datapack" => ResourceType::DataPack,
        _ => ResourceType::Mod,
    });

    let sort = args.sort.as_deref().map(|s| match s {
        "downloads" => SortOrder::Downloads,
        "updated" => SortOrder::Updated,
        "newest" => SortOrder::Newest,
        "follows" => SortOrder::Follows,
        _ => SortOrder::Relevance,
    });

    let req = UnifiedSearchRequest {
        query: args.query,
        source,
        resource_type,
        game_version: args.game_version,
        loader: args.loader,
        sort,
        offset: args.offset,
        limit: args.limit,
    };

    let resp = platforms::search_unified(&req)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&resp).map_err(|e| e.to_string())
}

/// Get versions for a project from a specific source
#[command]
pub async fn get_mod_versions_unified(
    source: String,
    project_id: String,
    game_version: Option<String>,
    loader: Option<String>,
) -> Result<String, String> {
    let src = match source.as_str() {
        "modrinth" => ModSource::Modrinth,
        "curseforge" => ModSource::CurseForge,
        _ => return Err("Invalid source".to_string()),
    };

    let versions =
        platforms::get_project_versions(&src, &project_id, game_version.as_deref(), loader.as_deref())
            .await
            .map_err(|e| e.to_string())?;

    serde_json::to_string(&versions).map_err(|e| e.to_string())
}

/// Get project details from a specific source
#[command]
pub async fn get_mod_details_unified(
    source: String,
    project_id: String,
) -> Result<String, String> {
    let src = match source.as_str() {
        "modrinth" => ModSource::Modrinth,
        "curseforge" => ModSource::CurseForge,
        _ => return Err("Invalid source".to_string()),
    };

    let details = platforms::get_project_details(&src, &project_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&details).map_err(|e| e.to_string())
}
