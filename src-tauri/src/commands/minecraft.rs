use crate::api::aggregator;
use crate::api::minecraft;
use crate::api::modrinth;
use crate::api::curseforge;
use crate::db;
use crate::utils::launcher;
use crate::utils::sharing;
use crate::AppState;
use serde::Serialize;
use tauri::State;

use super::instances::InstanceListItem;

#[derive(Serialize)]
pub struct VersionEntry {
    pub id: String,
    pub version_type: String,
    pub release_time: String,
}

#[tauri::command]
pub async fn get_version_manifest() -> Result<Vec<VersionEntry>, String> {
    let manifest = minecraft::fetch_version_manifest()
        .await
        .map_err(|e| e.to_string())?;

    Ok(manifest
        .versions
        .into_iter()
        .map(|v| VersionEntry {
            id: v.id,
            version_type: v.version_type,
            release_time: v.release_time,
        })
        .collect())
}

#[tauri::command]
pub async fn prepare_instance(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<String, String> {
    let instance = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?
    };

    let base_dir = crate::utils::paths::data_dir();
    let java_path = launcher::find_java(None).map_err(|e| e.to_string())?;
    let game_launcher = launcher::GameLauncher::new(base_dir, java_path);

    game_launcher
        .prepare(&instance)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Instance {} prepared successfully", instance.name))
}

#[tauri::command]
pub async fn launch_game(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<u32, String> {
    // Get instance and account
    let (instance, account) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let instance = db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?;

        let account = db::accounts::get_active_account(&db)
            .map_err(|e| e.to_string())?
            .ok_or("No account logged in. Please sign in first.")?;

        // Record play
        db::instances::record_play(&db, &instance_id).map_err(|e| e.to_string())?;

        (instance, account)
    };

    let base_dir = crate::utils::paths::data_dir();
    let java_path = launcher::find_java(None).map_err(|e| e.to_string())?;
    let game_launcher = launcher::GameLauncher::new(base_dir, java_path);

    // Prepare (download) if needed
    game_launcher
        .prepare(&instance)
        .await
        .map_err(|e| e.to_string())?;

    // Launch
    let pid = game_launcher
        .launch(
            &instance,
            &account.access_token,
            &account.username,
            &account.uuid,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(pid)
}

#[derive(Serialize)]
pub struct JavaInfo {
    pub found: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub fn check_java() -> JavaInfo {
    match launcher::find_java(None) {
        Ok(path) => JavaInfo {
            found: true,
            path: Some(path.to_string_lossy().to_string()),
            error: None,
        },
        Err(e) => JavaInfo {
            found: false,
            path: None,
            error: Some(e.to_string()),
        },
    }
}

// ── Modrinth commands ──────────────────────────────────────────

#[derive(Serialize)]
pub struct ModSearchResult {
    pub source: String,
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub icon_url: String,
    pub downloads: u64,
    pub categories: Vec<String>,
}

#[tauri::command]
pub async fn modrinth_search(
    query: String,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<Vec<ModSearchResult>, String> {
    let results = modrinth::search(
        &query,
        None,
        offset.unwrap_or(0),
        limit.unwrap_or(20),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(results
        .hits
        .into_iter()
        .map(|h| ModSearchResult {
            source: "modrinth".to_string(),
            project_id: h.project_id,
            slug: h.slug,
            title: h.title,
            description: h.description,
            icon_url: h.icon_url,
            downloads: h.downloads,
            categories: h.display_categories.unwrap_or_default(),
        })
        .collect())
}

// ── CurseForge commands ────────────────────────────────────────

#[tauri::command]
pub async fn curseforge_search(
    state: State<'_, AppState>,
    query: String,
    offset: Option<i32>,
    limit: Option<i32>,
) -> Result<Vec<ModSearchResult>, String> {
    let api_key = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db::settings::get_curseforge_api_key(&db)
            .map_err(|e| e.to_string())?
            .ok_or("CurseForge API key not configured. Add it in Settings.")?
    };

    let results = curseforge::search_mods(
        &api_key,
        &query,
        None,
        None,
        offset.unwrap_or(0),
        limit.unwrap_or(20),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(results
        .data
        .into_iter()
        .map(|m| ModSearchResult {
            source: "curseforge".to_string(),
            project_id: m.id.to_string(),
            slug: m.slug,
            title: m.name,
            description: m.summary,
            icon_url: m.logo.and_then(|l| l.url).unwrap_or_default(),
            downloads: m.download_count,
            categories: m
                .categories
                .unwrap_or_default()
                .into_iter()
                .map(|c| c.name)
                .collect(),
        })
        .collect())
}

// ── Aggregated search (Modrinth + CurseForge) ─────────────────

#[derive(Serialize)]
pub struct AggregatedSearchResult {
    pub source: String,
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub icon_url: String,
    pub downloads: u64,
    pub categories: Vec<String>,
}

/// Search both Modrinth and CurseForge concurrently, merge and deduplicate.
#[tauri::command]
pub async fn aggregated_search(
    state: State<'_, AppState>,
    query: String,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<Vec<AggregatedSearchResult>, String> {
    let curseforge_key = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db::settings::get_curseforge_api_key(&db)
            .map_err(|e| e.to_string())
            .ok()
            .flatten()
    };

    let results = aggregator::aggregated_search(
        &query,
        offset.unwrap_or(0),
        limit.unwrap_or(20),
        curseforge_key.as_deref(),
        offset.unwrap_or(0) as i32,
        limit.unwrap_or(20) as i32,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(|r| AggregatedSearchResult {
            source: r.source,
            project_id: r.project_id,
            slug: r.slug,
            title: r.title,
            description: r.description,
            icon_url: r.icon_url,
            downloads: r.downloads,
            categories: r.categories,
        })
        .collect())
}

// ── Instance sharing ──────────────────────────────────────────

#[derive(Serialize)]
pub struct ShareCode {
    pub code: String,
    pub name: String,
    pub mod_count: usize,
}

#[derive(serde::Deserialize)]
pub struct ImportSharePayload {
    pub code: String,
}

/// Export an instance as a share code.
#[tauri::command]
pub async fn export_instance_share(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<ShareCode, String> {
    let (instance, mods) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let instance = db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?;

        let mods = db::mods::get_instance_mods(&db, &instance_id)
            .map_err(|e| e.to_string())?;

        (instance, mods)
    };

    let shared_mods: Vec<sharing::SharedMod> = mods
        .into_iter()
        .map(|m| sharing::SharedMod {
            source: m.source,
            project_id: m.mod_id,
            version_id: String::new(),
            name: m.name,
            file_name: m.file_name,
        })
        .collect();

    let mod_count = shared_mods.len();

    let code = sharing::export_instance(
        &instance.name,
        &instance.game_version,
        &instance.loader,
        &instance.loader_version.as_deref().unwrap_or(""),
        instance.allocated_memory_mb,
        instance.java_args.as_deref(),
        instance.resolution.as_deref(),
        instance.notes.as_deref(),
        shared_mods,
    )
    .map_err(|e| e.to_string())?;

    Ok(ShareCode {
        code,
        name: instance.name,
        mod_count,
    })
}

/// Import an instance from a share code.
#[tauri::command]
pub async fn import_instance_share(
    state: State<'_, AppState>,
    payload: ImportSharePayload,
) -> Result<InstanceListItem, String> {
    let parsed = sharing::import_instance(&payload.code).map_err(|e| e.to_string())?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let instance = db::instances::create_instance(
        &db,
        db::instances::CreateInstanceParams {
            name: parsed.name,
            game_version: parsed.game_version,
            loader: parsed.loader,
            loader_version: Some(parsed.loader_version).filter(|s| !s.is_empty()),
            icon: None,
            java_args: parsed.java_args,
            allocated_memory_mb: parsed.allocated_memory_mb,
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(InstanceListItem {
        id: instance.id,
        name: instance.name,
        game_version: instance.game_version,
        loader: instance.loader,
        loader_version: instance.loader_version,
        icon: instance.icon,
        created_at: instance.created_at,
        last_played: instance.last_played,
        play_time_secs: instance.play_time_secs,
        allocated_memory_mb: instance.allocated_memory_mb,
    })
}
