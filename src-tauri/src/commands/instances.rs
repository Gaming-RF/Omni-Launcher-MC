use crate::db;
use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct InstanceListItem {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub icon: Option<String>,
    pub created_at: String,
    pub last_played: Option<String>,
    pub play_time_secs: i64,
    pub allocated_memory_mb: i64,
}

#[derive(serde::Deserialize)]
pub struct CreateInstancePayload {
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub icon: Option<String>,
    pub java_args: Option<String>,
    pub allocated_memory_mb: i64,
}

#[tauri::command]
pub fn get_instances(state: State<'_, AppState>) -> Result<Vec<InstanceListItem>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let instances = db::instances::get_all_instances(&db).map_err(|e| e.to_string())?;

    Ok(instances
        .into_iter()
        .map(|i| InstanceListItem {
            id: i.id,
            name: i.name,
            game_version: i.game_version,
            loader: i.loader,
            loader_version: i.loader_version,
            icon: i.icon,
            created_at: i.created_at,
            last_played: i.last_played,
            play_time_secs: i.play_time_secs,
            allocated_memory_mb: i.allocated_memory_mb,
        })
        .collect())
}

#[tauri::command]
pub fn create_instance(
    state: State<'_, AppState>,
    payload: CreateInstancePayload,
) -> Result<InstanceListItem, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let instance = db::instances::create_instance(
        &db,
        db::instances::CreateInstanceParams {
            name: payload.name,
            game_version: payload.game_version,
            loader: payload.loader,
            loader_version: payload.loader_version,
            icon: payload.icon,
            java_args: payload.java_args,
            allocated_memory_mb: payload.allocated_memory_mb,
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

#[tauri::command]
pub fn delete_instance(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::instances::delete_instance(&db, &id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_instance(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    java_args: Option<String>,
    allocated_memory_mb: Option<i64>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut instance = db::instances::get_instance(&db, &id)
        .map_err(|e| e.to_string())?
        .ok_or("Instance not found")?;

    if let Some(n) = name {
        instance.name = n;
    }
    if let Some(j) = java_args {
        instance.java_args = Some(j);
    }
    if let Some(m) = allocated_memory_mb {
        instance.allocated_memory_mb = m;
    }

    db::instances::update_instance(&db, &instance).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
pub struct SettingsInfo {
    pub default_memory_mb: String,
    pub theme: String,
    pub language: String,
    pub java_path: Option<String>,
    pub curseforge_api_key: Option<String>,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsInfo, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(SettingsInfo {
        default_memory_mb: db::settings::get_setting(&db, "default_memory_mb")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "4096".to_string()),
        theme: db::settings::get_setting(&db, "theme")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "dark".to_string()),
        language: db::settings::get_setting(&db, "language")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "en".to_string()),
        java_path: db::settings::get_setting(&db, "java_path").map_err(|e| e.to_string())?,
        curseforge_api_key: db::settings::get_setting(&db, "curseforge_api_key")
            .map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
pub fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::settings::set_setting(&db, &key, &value).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn duplicate_instance(
    state: State<'_, AppState>,
    instance_id: String,
    new_name: String,
) -> Result<InstanceListItem, String> {
    let (original, mods) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let original = db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?;
        let mods = db::mods::get_instance_mods(&db, &instance_id).map_err(|e| e.to_string())?;
        (original, mods)
    };

    // Create new instance in DB with same settings
    let new_instance = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db::instances::create_instance(
            &db,
            db::instances::CreateInstanceParams {
                name: new_name,
                game_version: original.game_version,
                loader: original.loader,
                loader_version: original.loader_version,
                icon: original.icon,
                java_args: original.java_args,
                allocated_memory_mb: original.allocated_memory_mb,
            },
        )
        .map_err(|e| e.to_string())?
    };

    // Copy instance directory
    let base = crate::utils::paths::data_dir();
    let src = base.join("instances").join(&instance_id);
    let dst = base.join("instances").join(&new_instance.id);
    if src.exists() {
        copy_dir_all(&src, &dst).map_err(|e| e.to_string())?;
    }

    // Copy mods to DB
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for m in mods {
            db::mods::record_mod_install(
                &db,
                &new_instance.id,
                &m.mod_id,
                &m.source,
                &m.name,
                &m.version,
                &m.file_name,
            )
            .ok();
        }
    }

    Ok(InstanceListItem {
        id: new_instance.id,
        name: new_instance.name,
        game_version: new_instance.game_version,
        loader: new_instance.loader,
        loader_version: new_instance.loader_version,
        icon: new_instance.icon,
        created_at: new_instance.created_at,
        last_played: new_instance.last_played,
        play_time_secs: new_instance.play_time_secs,
        allocated_memory_mb: new_instance.allocated_memory_mb,
    })
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

// ── Instance Share ──────────────────────────────────────────────

use base64::{engine::general_purpose::STANDARD as B64, Engine};

#[derive(serde::Serialize, serde::Deserialize)]
struct SharePayload {
    name: String,
    game_version: String,
    loader: String,
    loader_version: Option<String>,
    icon: Option<String>,
    java_args: Option<String>,
    allocated_memory_mb: i64,
}

#[derive(Serialize)]
pub struct ShareCode {
    pub code: String,
    pub name: String,
    pub mod_count: usize,
}

#[derive(serde::Deserialize)]
pub struct ImportPayload {
    pub code: String,
}

/// Export an instance configuration as a shareable base64 code.
#[tauri::command]
pub fn export_instance_share(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<ShareCode, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let instance = db::instances::get_instance(&db, &instance_id)
        .map_err(|e| e.to_string())?
        .ok_or("Instance not found")?;

    let mods = db::mods::get_instance_mods(&db, &instance_id).map_err(|e| e.to_string())?;

    let share = SharePayload {
        name: instance.name.clone(),
        game_version: instance.game_version,
        loader: instance.loader,
        loader_version: instance.loader_version,
        icon: instance.icon,
        java_args: instance.java_args,
        allocated_memory_mb: instance.allocated_memory_mb,
    };

    let json = serde_json::to_string(&share).map_err(|e| e.to_string())?;
    let code = format!("OMC:{}", B64.encode(json.as_bytes()));

    Ok(ShareCode {
        code,
        name: instance.name,
        mod_count: mods.len(),
    })
}

/// Import an instance from a share code. Creates a new instance.
#[tauri::command]
pub fn import_instance_share(
    state: State<'_, AppState>,
    payload: ImportPayload,
) -> Result<InstanceListItem, String> {
    let code = payload
        .code
        .strip_prefix("OMC:")
        .ok_or("Invalid share code: must start with OMC:")?;

    let bytes = B64
        .decode(code)
        .map_err(|e| format!("Invalid share code: {}", e))?;
    let json = String::from_utf8(bytes).map_err(|e| format!("Invalid share code: {}", e))?;
    let share: SharePayload =
        serde_json::from_str(&json).map_err(|e| format!("Invalid share code: {}", e))?;

    // Validate the share data
    crate::utils::validate::validate_instance_name(&share.name)?;

    let suffix = " (imported)";
    let max_base = 64usize.saturating_sub(suffix.len());
    // Truncate at a char boundary to avoid panicking on multi-byte UTF-8
    let base_name = if share.name.len() > max_base {
        let mut end = max_base;
        while end > 0 && !share.name.is_char_boundary(end) {
            end -= 1;
        }
        &share.name[..end]
    } else {
        &share.name
    };
    let truncated_name = format!("{}{}", base_name, suffix);

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let instance = db::instances::create_instance(
        &db,
        db::instances::CreateInstanceParams {
            name: truncated_name,
            game_version: share.game_version,
            loader: share.loader,
            loader_version: share.loader_version,
            icon: share.icon,
            java_args: share.java_args,
            allocated_memory_mb: share.allocated_memory_mb,
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
