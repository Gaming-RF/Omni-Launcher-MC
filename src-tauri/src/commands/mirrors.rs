use crate::db::settings::{get_setting, set_setting};
use crate::utils::mirrors::{resolve_url, Mirror};
use crate::AppState;
use serde::Serialize;
use std::time::Instant;
use tauri::State;

#[derive(Serialize, Clone)]
pub struct MirrorInfo {
    id: String,
    name: String,
    base_url: String,
    is_active: bool,
    latency_ms: Option<u64>,
}

/// Build a `MirrorInfo` for a given mirror, marking it active if it matches
/// the currently-selected mirror id.
fn build_info(mirror: &Mirror, active_id: &str, latency_ms: Option<u64>) -> MirrorInfo {
    MirrorInfo {
        id: mirror.id().to_string(),
        name: mirror.name().to_string(),
        base_url: mirror.base_url().to_string(),
        is_active: mirror.id() == active_id,
        latency_ms,
    }
}

/// Return the active mirror id stored in settings (defaults to `"official"`).
fn active_mirror_id(db: &rusqlite::Connection) -> String {
    get_setting(db, "download_mirror")
        .ok()
        .flatten()
        .unwrap_or_else(|| "official".to_string())
}

// ── Tauri Commands ──────────────────────────────────────────────────────────

/// List all available mirrors with active state.
#[tauri::command]
pub fn list_mirrors(state: State<'_, AppState>) -> Result<Vec<MirrorInfo>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let active_id = active_mirror_id(&db);
    Ok(Mirror::all()
        .iter()
        .map(|m| build_info(m, &active_id, None))
        .collect())
}

/// Get the currently-selected mirror.
#[tauri::command]
pub fn get_mirror(state: State<'_, AppState>) -> Result<MirrorInfo, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let active_id = active_mirror_id(&db);
    let mirror = Mirror::from_id(&active_id).unwrap_or(Mirror::Official);
    Ok(build_info(&mirror, &active_id, None))
}

/// Set the active mirror by its id string.
#[tauri::command]
pub fn set_mirror(state: State<'_, AppState>, mirror_id: String) -> Result<(), String> {
    // Validate the id first
    if Mirror::from_id(&mirror_id).is_none() {
        return Err(format!("Unknown mirror id: {}", mirror_id));
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    set_setting(&db, "download_mirror", &mirror_id).map_err(|e| e.to_string())
}

/// Test a single mirror's latency by sending an HTTP HEAD request.
/// Returns latency in milliseconds.
#[tauri::command]
pub async fn test_mirror(
    state: State<'_, AppState>,
    mirror_id: String,
) -> Result<u64, String> {
    let mirror = Mirror::from_id(&mirror_id)
        .ok_or_else(|| format!("Unknown mirror id: {}", mirror_id))?;

    // Use a well-known Mojang URL resolved through the mirror as the test target.
    let test_url = resolve_url(
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
        &mirror,
    );

    // For Official, just hit Mojang directly; for mirrors, hit their resolved URL.
    let url = if test_url.is_empty() || mirror == Mirror::Official {
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json".to_string()
    } else {
        test_url
    };

    let start = Instant::now();
    let client = &state.http;
    let resp = client
        .head(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let _status = resp.status(); // consume the response
    let elapsed = start.elapsed().as_millis() as u64;
    Ok(elapsed)
}

/// Test all mirrors and return results with latency info.
#[tauri::command]
pub async fn test_all_mirrors(
    state: State<'_, AppState>,
) -> Result<Vec<MirrorInfo>, String> {
    let active_id = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        active_mirror_id(&db)
    };

    let mut results = Vec::new();
    for mirror in Mirror::all() {
        let latency = if mirror == Mirror::Official {
            // Test official Mojang endpoint
            let start = Instant::now();
            let res = state
                .http
                .head("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await;
            match res {
                Ok(_) => Some(start.elapsed().as_millis() as u64),
                Err(_) => None,
            }
        } else {
            let test_url = resolve_url(
                "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
                &mirror,
            );
            let start = Instant::now();
            let res = state
                .http
                .head(&test_url)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await;
            match res {
                Ok(_) => Some(start.elapsed().as_millis() as u64),
                Err(_) => None,
            }
        };
        results.push(build_info(&mirror, &active_id, latency));
    }

    Ok(results)
}

/// Resolve a Mojang download URL through the currently-selected mirror.
#[tauri::command]
pub fn resolve_download_url(
    state: State<'_, AppState>,
    url: String,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let active_id = active_mirror_id(&db);
    let mirror = Mirror::from_id(&active_id).unwrap_or(Mirror::Official);
    Ok(resolve_url(&url, &mirror))
}
