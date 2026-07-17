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
