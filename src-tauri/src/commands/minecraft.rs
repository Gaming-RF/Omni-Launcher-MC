use crate::error::AppError;
use crate::api::curseforge;
use crate::api::loaders;
use crate::api::minecraft;
use crate::api::modrinth;
use crate::commands::instances::InstanceListItem;
use crate::commands::loaders::ModVersionInfo;
use crate::db;
use crate::utils::launcher;
use crate::utils::progress;
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
pub async fn get_version_manifest(state: State<'_, AppState>) -> Result<Vec<VersionEntry>, AppError> {
    let manifest = minecraft::fetch_version_manifest(Some(&state.http))
        .await
        ?;

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
) -> Result<String, AppError> {
    let instance = {
        let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::instances::get_instance(&db, &instance_id)
            ?
            .ok_or("Instance not found")?
    };

    let task_id = format!("prepare-{}", instance_id);

    // Emit progress start
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(
                app,
                &task_id,
                "starting",
                &format!("Preparing {}...", instance.name),
            );
        }
    }

    let base_dir = crate::utils::paths::data_dir();

    // Use ensure_java for auto-download
    let java_path = crate::utils::java::ensure_java(&instance.game_version, None)
        .await
        ?;

    let game_launcher = launcher::GameLauncher::new(base_dir, java_path, state.http.clone());

    // Emit progress: downloading version JSON
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(app, &task_id, "version_json", "Downloading version JSON...");
        }
    }

    game_launcher
        .prepare(&instance)
        .await
        ?;

    // Emit completion
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::complete(app, &task_id, &format!("{} ready to play!", instance.name));
        }
    }

    Ok(format!("Instance {} prepared successfully", instance.name))
}

#[tauri::command]
pub async fn launch_game(state: State<'_, AppState>, instance_id: String) -> Result<u32, AppError> {
    let task_id = format!("launch-{}", instance_id);

    // Emit progress: starting
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(app, &task_id, "starting", "Preparing to launch...");
        }
    }

    // Get instance and account
    let (instance, account) = {
        let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let instance = db::instances::get_instance(&db, &instance_id)
            ?
            .ok_or("Instance not found")?;

        let account = db::accounts::get_active_account(&db)
            ?
            .ok_or("No account logged in. Please sign in first.")?;

        db::instances::record_play(&db, &instance_id)?;

        (instance, account)
    };

    let base_dir = crate::utils::paths::data_dir();

    // Auto-download Java if needed
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(app, &task_id, "java", "Checking Java...");
        }
    }

    let java_path = crate::utils::java::ensure_java(&instance.game_version, None)
        .await
        ?;

    let game_launcher = launcher::GameLauncher::new(base_dir, java_path, state.http.clone());

    // Prepare (download assets if needed)
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(app, &task_id, "assets", "Downloading game files...");
        }
    }

    game_launcher
        .prepare(&instance)
        .await
        ?;

    // Launch
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(app, &task_id, "launching", "Starting Minecraft...");
        }
    }

    let (pid, child) = game_launcher
        .launch(
            &instance,
            &account.access_token,
            &account.username,
            &account.uuid,
            false,
        )
        .await
        ?;

    // Register the child process with the process manager
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            state.process_manager.spawn(app, &instance_id, child, pid);
        }
    }

    // Emit completion
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::complete(app, &task_id, &format!("Minecraft launched (PID {})", pid));
        }
    }

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
) -> Result<Vec<ModSearchResult>, AppError> {
    let query = crate::utils::validate::sanitize_query(&query);
    if query.is_empty() {
        return Err(AppError::Internal("Search query cannot be empty".to_string()));
    }
    let results = modrinth::search(&query, None, offset.unwrap_or(0), limit.unwrap_or(20))
        .await
        ?;

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
) -> Result<Vec<ModSearchResult>, AppError> {
    let query = crate::utils::validate::sanitize_query(&query);
    if query.is_empty() {
        return Err(AppError::Internal("Search query cannot be empty".to_string()));
    }

    let api_key = {
        let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::settings::get_curseforge_api_key(&db)
            ?
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
    ?;

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
            downloads: m.download_count.max(0) as u64,
            categories: m
                .categories
                .unwrap_or_default()
                .into_iter()
                .map(|c| c.name)
                .collect(),
        })
        .collect())
}

// ── Modpack Search ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct ModpackSearchResult {
    pub source: String,
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub icon_url: String,
    pub downloads: u64,
    pub categories: Vec<String>,
    pub game_versions: Vec<String>,
}

/// Search Modrinth for modpacks specifically.
#[tauri::command]
pub async fn search_modpacks_modrinth(
    query: String,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<Vec<ModpackSearchResult>, AppError> {
    let query = crate::utils::validate::sanitize_query(&query);
    let facets = r#"[["project_type:modpack"]]"#;
    let results = modrinth::search(
        &query,
        Some(facets),
        offset.unwrap_or(0),
        limit.unwrap_or(20),
    )
    .await
    ?;

    Ok(results
        .hits
        .into_iter()
        .map(|h| ModpackSearchResult {
            source: "modrinth".to_string(),
            project_id: h.project_id,
            slug: h.slug,
            title: h.title,
            description: h.description,
            icon_url: h.icon_url,
            downloads: h.downloads,
            categories: h.display_categories.unwrap_or_default(),
            game_versions: h.versions,
        })
        .collect())
}

/// Search CurseForge for modpacks specifically (classId=4471).
#[tauri::command]
pub async fn search_modpacks_curseforge(
    state: State<'_, AppState>,
    query: String,
    offset: Option<i32>,
    limit: Option<i32>,
) -> Result<Vec<ModpackSearchResult>, AppError> {
    let query = crate::utils::validate::sanitize_query(&query);

    let api_key = {
        let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::settings::get_curseforge_api_key(&db)
            ?
            .ok_or("CurseForge API key not configured. Add it in Settings.")?
    };

    let client = reqwest::Client::new();
    let url = format!(
        "{}/v1/mods/search?gameId={}&classId=4471&searchFilter={}&index={}&pageSize={}",
        crate::api::curseforge::BASE_URL,
        crate::api::curseforge::MINECRAFT_GAME_ID,
        urlencoding::encode(&query),
        offset.unwrap_or(0),
        limit.unwrap_or(20),
    );

    let resp: crate::api::curseforge::SearchResponse = client
        .get(&url)
        .header("x-api-key", &api_key)
        .header("Accept", "application/json")
        .send()
        .await
        ?
        .json()
        .await
        ?;

    Ok(resp
        .data
        .into_iter()
        .map(|m| {
            let game_versions = m
                .latest_files
                .as_ref()
                .map(|files| {
                    files
                        .iter()
                        .flat_map(|f| f.game_versions.clone())
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default();
            ModpackSearchResult {
                source: "curseforge".to_string(),
                project_id: m.id.to_string(),
                slug: m.slug,
                title: m.name,
                description: m.summary,
                icon_url: m.logo.and_then(|l| l.url).unwrap_or_default(),
                downloads: m.download_count.max(0) as u64,
                categories: m
                    .categories
                    .unwrap_or_default()
                    .into_iter()
                    .map(|c| c.name)
                    .collect(),
                game_versions,
            }
        })
        .collect())
}

/// Get available versions for a Modrinth modpack.
#[tauri::command]
pub async fn get_modpack_versions_modrinth(
    project_id: String,
) -> Result<Vec<ModVersionInfo>, AppError> {
    let versions = modrinth::get_project_versions(&project_id, None, None)
        .await
        ?;

    Ok(versions
        .into_iter()
        .map(|v| ModVersionInfo {
            version_id: v.id,
            name: v.name,
            version_number: v.version_number,
            date_published: v.date_published,
            download_count: v.downloads as i64,
            file_name: v.files.first().map(|f| f.filename.clone()),
            file_url: v.files.first().map(|f| f.url.clone()),
            game_versions: v.game_versions,
        })
        .collect())
}

/// Download a modpack from URL, parse, create instance, install everything.
/// This is the one-click modpack install command.
#[tauri::command]
pub async fn download_and_install_modpack(
    state: State<'_, AppState>,
    download_url: String,
    source: String,
    name: String,
) -> Result<InstanceListItem, AppError> {
    let task_id = format!("modpack-{}", uuid::Uuid::new_v4());

    // Emit progress: downloading
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(
                app,
                &task_id,
                "modpack",
                &format!("Downloading {}...", name),
            );
        }
    }

    // Download the modpack file
    let temp_dir = std::env::temp_dir().join("omc-modpacks");
    std::fs::create_dir_all(&temp_dir)?;

    let ext = if download_url.ends_with(".mrpack") {
        ".mrpack"
    } else {
        ".zip"
    };
    let temp_path = temp_dir.join(format!("{}{}", uuid::Uuid::new_v4(), ext));

    let bytes = state
        .http
        .get(&download_url)
        .send()
        .await
        ?
        .bytes()
        .await
        ?;

    std::fs::write(&temp_path, &bytes)?;

    // Emit progress: parsing
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(app, &task_id, "modpack", "Parsing modpack...");
        }
    }

    // Parse the modpack
    let info = if source == "modrinth" {
        crate::utils::modpack::parse_mrpack(&temp_path)
    } else {
        crate::utils::modpack::parse_cf_modpack(&temp_path)
    }
    ?;

    // Create instance
    let instance = {
        let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let display_name = if name.len() > 60 { &name[..60] } else { &name };
        db::instances::create_instance(
            &db,
            db::instances::CreateInstanceParams {
                name: display_name.to_string(),
                game_version: info.game_version.clone(),
                loader: info.loader.clone(),
                loader_version: Some(info.loader_version.clone()),
                icon: None,
                java_args: None,
                allocated_memory_mb: 4096,
            },
        )
        ?
    };

    let instance_dir = crate::utils::paths::instances_dir().join(&instance.id);

    // Emit progress: installing
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(
                app,
                &task_id,
                "modpack",
                &format!("Installing {} mods...", info.file_count),
            );
        }
    }

    // Install modpack files
    if source == "modrinth" {
        crate::utils::modpack::install_mrpack(&temp_path, &instance_dir, &state.http)
            .await
            ?;
    } else {
        let (_info, cf_files) =
            crate::utils::modpack::install_cf_modpack(&temp_path, &instance_dir)
                .await
                ?;

        // Download CurseForge mods via API
        let api_key = {
            let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
            db::settings::get_curseforge_api_key(&db)
                ?
                .unwrap_or_default()
        };

        if !api_key.is_empty() {
            let mods_dir = instance_dir.join("mods");
            std::fs::create_dir_all(&mods_dir)?;
            for cf_file in &cf_files {
                if let Ok(Some(url)) =
                    curseforge::get_file_download_url(&api_key, cf_file.project_id, cf_file.file_id)
                        .await
                {
                    let filename = url.rsplit('/').next().unwrap_or("mod.jar");
                    let dest = mods_dir.join(filename);
                    let _ = minecraft::download_file(Some(&state.http), &url, &dest).await;
                }
            }
        }
    }

    // Emit progress: installing loader
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(
                app,
                &task_id,
                "loader",
                &format!("Installing {} loader...", info.loader),
            );
        }
    }

    // Install the mod loader
    let base_dir = crate::utils::paths::data_dir();
    if info.loader != "vanilla" && !info.loader_version.is_empty() {
        let result = match info.loader.as_str() {
            "fabric" => {
                loaders::fabric::install(&base_dir, &info.game_version, &info.loader_version).await
            }
            "quilt" => {
                loaders::quilt::install(&base_dir, &info.game_version, &info.loader_version).await
            }
            "forge" => {
                loaders::forge::install(&base_dir, &info.game_version, &info.loader_version).await
            }
            "neoforge" => {
                loaders::neoforge::install(&base_dir, &info.game_version, &info.loader_version)
                    .await
            }
            _ => Ok(String::new()),
        };
        if let Err(e) = result {
            log::warn!("Loader install warning: {}", e);
        }
    }

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    // Emit completion
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::complete(app, &task_id, &format!("{} installed successfully!", name));
        }
    }

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

// ── Resource Packs & Shaders ───────────────────────────────────

#[derive(Serialize)]
pub struct InstalledPackInfo {
    pub file_name: String,
    pub enabled: bool,
}

/// List installed resource packs or shader packs for an instance.
/// `pack_type` is "resourcepacks" or "shaderpacks".
#[tauri::command]
pub fn list_installed_packs(
    instance_id: String,
    pack_type: String,
) -> Result<Vec<InstalledPackInfo>, AppError> {
    let dir = match pack_type.as_str() {
        "resourcepacks" | "shaderpacks" => crate::utils::paths::instances_dir()
            .join(&instance_id)
            .join(&pack_type),
        _ => return Err(AppError::Internal(format!("Invalid pack type: {}", pack_type))),
    };

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries: Vec<InstalledPackInfo> = std::fs::read_dir(&dir)
        ?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip directories and hidden files
            if entry.path().is_dir() || name.starts_with('.') {
                return None;
            }
            let enabled = !name.ends_with(".disabled");
            Some(InstalledPackInfo {
                file_name: name,
                enabled,
            })
        })
        .collect();

    Ok(entries)
}

/// Toggle a resource pack or shader pack enabled/disabled.
#[tauri::command]
pub fn toggle_pack(
    instance_id: String,
    pack_type: String,
    file_name: String,
) -> Result<bool, AppError> {
    let dir = match pack_type.as_str() {
        "resourcepacks" | "shaderpacks" => crate::utils::paths::instances_dir()
            .join(&instance_id)
            .join(&pack_type),
        _ => return Err(AppError::Internal(format!("Invalid pack type: {}", pack_type))),
    };

    let current_path = dir.join(&file_name);
    let is_disabled = file_name.ends_with(".disabled");

    let (new_name, new_enabled) = if is_disabled {
        (file_name.replace(".disabled", ""), true)
    } else {
        (format!("{}.disabled", file_name), false)
    };

    let new_path = dir.join(&new_name);

    if current_path.exists() {
        std::fs::rename(&current_path, &new_path)?;
    }

    Ok(new_enabled)
}

/// Delete a resource pack or shader pack.
#[tauri::command]
pub fn delete_pack(
    instance_id: String,
    pack_type: String,
    file_name: String,
) -> Result<(), AppError> {
    let path = match pack_type.as_str() {
        "resourcepacks" | "shaderpacks" => crate::utils::paths::instances_dir()
            .join(&instance_id)
            .join(&pack_type)
            .join(&file_name),
        _ => return Err(AppError::Internal(format!("Invalid pack type: {}", pack_type))),
    };

    if path.exists() {
        std::fs::remove_file(&path)?;
    }

    Ok(())
}

// ── Offline Launch ──────────────────────────────────────────────

/// Generate a deterministic offline UUID from a username.
/// Matches the standard Minecraft offline UUID: MD5 of "OfflinePlayer:{name}".
fn offline_uuid(username: &str) -> String {
    let digest = md5::compute(format!("OfflinePlayer:{}", username).as_bytes());
    let hex = format!("{:x}", digest);
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Launch a game instance in offline mode — no Microsoft account required.
/// Just provide a username and it launches.
#[tauri::command]
pub async fn launch_game_offline(
    state: State<'_, AppState>,
    instance_id: String,
    username: String,
) -> Result<u32, AppError> {
    crate::utils::validate::validate_id(&instance_id)?;
    crate::utils::validate::validate_username(&username)?;

    let task_id = format!("launch-{}", instance_id);

    // Emit progress: starting
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(
                app,
                &task_id,
                "starting",
                "Preparing to launch (offline)...",
            );
        }
    }

    let instance = {
        let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let inst = db::instances::get_instance(&db, &instance_id)
            ?
            .ok_or("Instance not found")?;

        db::instances::record_play(&db, &instance_id)?;

        inst
    };

    let base_dir = crate::utils::paths::data_dir();

    // Auto-download Java
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(app, &task_id, "java", "Checking Java...");
        }
    }

    let java_path = crate::utils::java::ensure_java(&instance.game_version, None)
        .await
        ?;

    let game_launcher = launcher::GameLauncher::new(base_dir, java_path, state.http.clone());

    // Prepare (download assets if needed)
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(app, &task_id, "assets", "Downloading game files...");
        }
    }

    game_launcher
        .prepare(&instance)
        .await
        ?;

    // Launch with offline credentials
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::phase_start(
                app,
                &task_id,
                "launching",
                "Starting Minecraft (offline)...",
            );
        }
    }

    let uuid = offline_uuid(&username);
    let access_token = "0"; // Dummy token for offline mode

    let (pid, child) = game_launcher
        .launch(&instance, access_token, &username, &uuid, true)
        .await
        ?;

    // Register with process manager
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            state.process_manager.spawn(app, &instance_id, child, pid);
        }
    }

    // Emit completion
    {
        let handle_guard = state.app_handle.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(app) = handle_guard.as_ref() {
            progress::complete(
                app,
                &task_id,
                &format!("Minecraft launched as {} (PID {})", username, pid),
            );
        }
    }

    Ok(pid)
}

// ── Aggregated Search ──────────────────────────────────────────

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

/// Search both Modrinth and CurseForge simultaneously, merge results.
#[tauri::command]
pub async fn aggregated_search(
    state: State<'_, AppState>,
    query: String,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<Vec<AggregatedSearchResult>, AppError> {
    let query = crate::utils::validate::sanitize_query(&query);
    if query.is_empty() {
        return Err(AppError::Internal("Search query cannot be empty".to_string()));
    }

    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(20);

    // Run both searches in parallel
    let modrinth_fut = modrinth::search(&query, None, offset, limit);
    let curseforge_fut = async {
        let api_key = {
            let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
            db::settings::get_curseforge_api_key(&db)?
        };
        Ok::<_, AppError>(match api_key {
            Some(key) => {
                curseforge::search_mods(&key, &query, None, None, offset as i32, limit as i32)
                    .await
                    .map(|r| r.data)
                    .map_err(|e| AppError::Internal(e.to_string()))?
            }
            None => vec![], // No API key = skip CurseForge
        })
    };

    let (modrinth_result, curseforge_result) = tokio::join!(modrinth_fut, curseforge_fut);

    let mut results: Vec<AggregatedSearchResult> = Vec::new();

    // Add Modrinth results
    if let Ok(mr) = modrinth_result {
        for h in mr.hits {
            results.push(AggregatedSearchResult {
                source: "modrinth".to_string(),
                project_id: h.project_id,
                slug: h.slug,
                title: h.title,
                description: h.description,
                icon_url: h.icon_url,
                downloads: h.downloads,
                categories: h.display_categories.unwrap_or_default(),
            });
        }
    }

    // Add CurseForge results
    if let Ok(cf) = curseforge_result {
        for m in cf {
            results.push(AggregatedSearchResult {
                source: "curseforge".to_string(),
                project_id: m.id.to_string(),
                slug: m.slug,
                title: m.name,
                description: m.summary,
                icon_url: m.logo.and_then(|l| l.url).unwrap_or_default(),
                downloads: m.download_count.max(0) as u64,
                categories: m
                    .categories
                    .unwrap_or_default()
                    .into_iter()
                    .map(|c| c.name)
                    .collect(),
            });
        }
    }

    Ok(results)
}
