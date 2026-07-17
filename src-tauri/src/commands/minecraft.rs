use crate::api::minecraft;
use crate::api::modrinth;
use crate::api::curseforge;
use crate::db;
use crate::utils::launcher;
use crate::AppState;
use serde::Serialize;
use tauri::State;

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
