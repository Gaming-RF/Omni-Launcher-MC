use crate::api::loaders;
use crate::api::minecraft;
use crate::api::modrinth;
use crate::db;
use crate::AppState;
use serde::Serialize;
use tauri::State;

// ── Loader Version Queries ─────────────────────────────────────

#[derive(Serialize)]
pub struct LoaderVersionInfo {
    pub version: String,
    pub stable: bool,
}

#[tauri::command]
pub async fn get_fabric_loader_versions(
    mc_version: String,
) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = loaders::fabric::get_loader_versions(&mc_version)
        .await
        .map_err(|e| e.to_string())?;

    Ok(versions
        .into_iter()
        .map(|v| LoaderVersionInfo {
            version: v.version,
            stable: v.stable,
        })
        .collect())
}

#[tauri::command]
pub async fn get_quilt_loader_versions(
    mc_version: String,
) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = loaders::quilt::get_loader_versions(&mc_version)
        .await
        .map_err(|e| e.to_string())?;

    Ok(versions
        .into_iter()
        .map(|v| LoaderVersionInfo {
            version: v.version,
            stable: v.stable,
        })
        .collect())
}

#[tauri::command]
pub async fn get_forge_versions(mc_version: String) -> Result<Vec<String>, String> {
    loaders::forge::get_forge_versions(&mc_version)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_neoforge_versions(mc_version: String) -> Result<Vec<String>, String> {
    loaders::neoforge::get_neoforge_versions(&mc_version)
        .await
        .map_err(|e| e.to_string())
}

// ── Loader Installation ────────────────────────────────────────

#[tauri::command]
pub async fn install_fabric_loader(
    state: State<'_, AppState>,
    instance_id: String,
    loader_version: String,
) -> Result<String, String> {
    let instance = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?
    };

    let base_dir = crate::utils::paths::data_dir();
    let profile_id = loaders::fabric::install(&base_dir, &instance.game_version, &loader_version)
        .await
        .map_err(|e| e.to_string())?;

    // Update the instance's loader and version info
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut inst = db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .unwrap();
        inst.loader = "fabric".to_string();
        inst.loader_version = Some(loader_version);
        // Update game_version to the profile ID so the launcher uses the Fabric JSON
        db::instances::update_instance(&db, &inst).map_err(|e| e.to_string())?;
    }

    Ok(format!("Fabric installed: {}", profile_id))
}

#[tauri::command]
pub async fn install_quilt_loader(
    state: State<'_, AppState>,
    instance_id: String,
    loader_version: String,
) -> Result<String, String> {
    let instance = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?
    };

    let base_dir = crate::utils::paths::data_dir();
    let profile_id = loaders::quilt::install(&base_dir, &instance.game_version, &loader_version)
        .await
        .map_err(|e| e.to_string())?;

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut inst = db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .unwrap();
        inst.loader = "quilt".to_string();
        inst.loader_version = Some(loader_version);
        db::instances::update_instance(&db, &inst).map_err(|e| e.to_string())?;
    }

    Ok(format!("Quilt installed: {}", profile_id))
}

#[tauri::command]
pub async fn install_forge_loader(
    state: State<'_, AppState>,
    instance_id: String,
    forge_version: String,
) -> Result<String, String> {
    let instance = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?
    };

    let base_dir = crate::utils::paths::data_dir();
    let profile_id = loaders::forge::install(&base_dir, &instance.game_version, &forge_version)
        .await
        .map_err(|e| e.to_string())?;

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut inst = db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .unwrap();
        inst.loader = "forge".to_string();
        inst.loader_version = Some(forge_version);
        db::instances::update_instance(&db, &inst).map_err(|e| e.to_string())?;
    }

    Ok(format!("Forge installed: {}", profile_id))
}

#[tauri::command]
pub async fn install_neoforge_loader(
    state: State<'_, AppState>,
    instance_id: String,
    neoforge_version: String,
) -> Result<String, String> {
    let instance = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?
    };

    let base_dir = crate::utils::paths::data_dir();
    let profile_id =
        loaders::neoforge::install(&base_dir, &instance.game_version, &neoforge_version)
            .await
            .map_err(|e| e.to_string())?;

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut inst = db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .unwrap();
        inst.loader = "neoforge".to_string();
        inst.loader_version = Some(neoforge_version);
        db::instances::update_instance(&db, &inst).map_err(|e| e.to_string())?;
    }

    Ok(format!("NeoForge installed: {}", profile_id))
}

// ── Per-Instance Mod Management ────────────────────────────────

#[derive(Serialize)]
pub struct InstalledModInfo {
    pub id: i64,
    pub mod_id: String,
    pub source: String,
    pub name: String,
    pub version: String,
    pub file_name: String,
    pub enabled: bool,
    pub installed_at: String,
}

#[tauri::command]
pub fn get_instance_mods(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<InstalledModInfo>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mods = db::mods::get_instance_mods(&db, &instance_id).map_err(|e| e.to_string())?;

    Ok(mods
        .into_iter()
        .map(|m| InstalledModInfo {
            id: m.id,
            mod_id: m.mod_id,
            source: m.source,
            name: m.name,
            version: m.version,
            file_name: m.file_name,
            enabled: m.enabled,
            installed_at: m.installed_at,
        })
        .collect())
}

/// Install a mod from Modrinth into an instance's mods/ directory.
#[tauri::command]
pub async fn install_mod_from_modrinth(
    state: State<'_, AppState>,
    instance_id: String,
    project_id: String,
    game_version: String,
    loader: String,
) -> Result<InstalledModInfo, String> {
    // Check if already installed
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if db::mods::is_mod_installed(&db, &instance_id, &project_id, "modrinth")
            .map_err(|e| e.to_string())?
        {
            return Err("Mod is already installed".to_string());
        }
    }

    // Get the latest version for this game version + loader
    let versions = modrinth::get_project_versions(
        &project_id,
        Some(&loader),
        Some(&game_version),
    )
    .await
    .map_err(|e| e.to_string())?;

    let version = versions
        .first()
        .ok_or("No compatible version found for this MC version and loader")?;

    let file = version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or("No downloadable files for this version")?;

    // Download the mod JAR to the instance's mods/ directory
    let mods_dir = crate::utils::paths::instances_dir()
        .join(&instance_id)
        .join("mods");
    std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;

    let dest = mods_dir.join(&file.filename);
    crate::api::minecraft::download_file(&file.url, &dest)
        .await
        .map_err(|e| e.to_string())?;

    // Record in database
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::mods::record_mod_install(
        &db,
        &instance_id,
        &project_id,
        "modrinth",
        &version.name,
        &version.version_number,
        &file.filename,
    )
    .map_err(|e| e.to_string())?;

    Ok(InstalledModInfo {
        id: 0, // Will be set by DB
        mod_id: project_id,
        source: "modrinth".to_string(),
        name: version.name.clone(),
        version: version.version_number.clone(),
        file_name: file.filename.clone(),
        enabled: true,
        installed_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Toggle a mod on/off (renames file with .disabled suffix).
#[tauri::command]
pub fn toggle_mod_enabled(
    state: State<'_, AppState>,
    mod_id: i64,
    instance_id: String,
) -> Result<bool, String> {
    let mods_dir = crate::utils::paths::instances_dir()
        .join(&instance_id)
        .join("mods");

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::mods::toggle_mod(&db, mod_id, &mods_dir).map_err(|e| e.to_string())
}

/// Remove a mod from an instance.
#[tauri::command]
pub fn remove_mod(
    state: State<'_, AppState>,
    mod_id: i64,
    instance_id: String,
) -> Result<(), String> {
    let mods_dir = crate::utils::paths::instances_dir()
        .join(&instance_id)
        .join("mods");

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::mods::remove_mod(&db, mod_id, &mods_dir).map_err(|e| e.to_string())
}

// ── Modpack Import ─────────────────────────────────────────────

use crate::utils::modpack;

#[derive(Serialize)]
pub struct ModpackInfoResult {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub game_version: String,
    pub loader: String,
    pub loader_version: String,
    pub file_count: usize,
}

/// Parse a .mrpack file and return its metadata.
#[tauri::command]
pub fn parse_mrpack_file(file_path: String) -> Result<ModpackInfoResult, String> {
    let path = std::path::PathBuf::from(&file_path);
    let info = modpack::parse_mrpack(&path).map_err(|e| e.to_string())?;

    Ok(ModpackInfoResult {
        name: info.name,
        version: info.version,
        summary: info.summary,
        game_version: info.game_version,
        loader: info.loader,
        loader_version: info.loader_version,
        file_count: info.file_count,
    })
}

/// Parse a CurseForge modpack ZIP and return its metadata.
#[tauri::command]
pub fn parse_cf_modpack_file(file_path: String) -> Result<ModpackInfoResult, String> {
    let path = std::path::PathBuf::from(&file_path);
    let info = modpack::parse_cf_modpack(&path).map_err(|e| e.to_string())?;

    Ok(ModpackInfoResult {
        name: info.name,
        version: info.version,
        summary: info.summary,
        game_version: info.game_version,
        loader: info.loader,
        loader_version: info.loader_version,
        file_count: info.file_count,
    })
}
