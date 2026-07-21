use crate::db;
use crate::AppState;
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

/// Supported launcher types for import.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherType {
    MultiMC,
    CurseForgeApp,
    PrismLauncher,
    ATLauncher,
    GDLauncher,
    Vanilla,
}

/// A discovered instance from another launcher that can be imported.
#[derive(Debug, Clone, Serialize)]
pub struct ImportableInstance {
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub source_path: String,
    pub source_launcher: String,
    pub icon: Option<String>,
}

/// Scan for instances from another launcher.
#[tauri::command]
pub async fn scan_launcher_instances(
    launcher_type: LauncherType,
    base_path: Option<String>,
) -> Result<Vec<ImportableInstance>, String> {
    let path = match base_path {
        Some(p) => PathBuf::from(p),
        None => detect_launcher_path(&launcher_type),
    };

    if !path.exists() {
        return Ok(vec![]);
    }

    match launcher_type {
        LauncherType::MultiMC | LauncherType::PrismLauncher => scan_multimc(&path).await,
        LauncherType::CurseForgeApp => scan_curseforge_app(&path).await,
        LauncherType::ATLauncher => scan_atlauncher(&path).await,
        LauncherType::GDLauncher => scan_gdlauncher(&path).await,
        LauncherType::Vanilla => scan_vanilla(&path).await,
    }
}

/// Import a specific instance into OmniLauncherMC.
#[tauri::command]
pub async fn import_launcher_instance(
    state: State<'_, AppState>,
    launcher_type: LauncherType,
    source_path: String,
    name: String,
    game_version: String,
    loader: String,
    loader_version: Option<String>,
) -> Result<db::instances::GameInstance, String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err(format!("Source path does not exist: {}", source_path));
    }

    // Create the instance in our DB
    let instance_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let instance = db::instances::GameInstance {
        id: instance_id.clone(),
        name: name.clone(),
        game_version: game_version.clone(),
        loader: loader.clone(),
        loader_version: loader_version.clone(),
        icon: None,
        created_at: now.clone(),
        last_played: None,
        play_time_secs: 0,
        java_args: None,
        resolution: None,
        notes: Some(format!("Imported from {:?}: {}", launcher_type, source_path)),
        groups: None,
        allocated_memory_mb: 4096,
        java_installation_id: None,
        pre_launch_cmd: None,
        post_exit_cmd: None,
        hook_env_vars: None,
    };

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db::instances::insert_instance(&db, &instance).map_err(|e| e.to_string())?;
    }

    // Copy mods/saves/resourcepacks from source to our instance
    let instance_dir = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id);
    tokio::fs::create_dir_all(&instance_dir)
        .await
        .map_err(|e| e.to_string())?;

    // Copy common directories
    let dirs_to_copy = ["mods", "saves", "resourcepacks", "shaderpacks", "config"];
    for dir_name in &dirs_to_copy {
        let src_dir = source.join(dir_name);
        if src_dir.exists() {
            let dst_dir = instance_dir.join(dir_name);
            copy_dir_recursive(&src_dir, &dst_dir)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    // Copy instance-specific files
    let files_to_copy = ["options.txt", "servers.dat"];
    for file_name in &files_to_copy {
        let src_file = source.join(file_name);
        if src_file.exists() {
            let dst_file = instance_dir.join(file_name);
            tokio::fs::copy(&src_file, &dst_file)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(instance)
}

/// Auto-detect the default path for a given launcher.
fn detect_launcher_path(launcher_type: &LauncherType) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();

    match launcher_type {
        LauncherType::MultiMC => {
            // Try common paths
            let candidates = [
                home.join("MultiMC"),
                home.join(".local/share/multimc"),
                PathBuf::from("/opt/multimc"),
            ];
            candidates
                .into_iter()
                .find(|p| p.exists())
                .unwrap_or(home.join("MultiMC"))
        }
        LauncherType::PrismLauncher => {
            let candidates = [
                home.join(".local/share/PrismLauncher"),
                home.join("PrismLauncher"),
            ];
            candidates
                .into_iter()
                .find(|p| p.exists())
                .unwrap_or(home.join(".local/share/PrismLauncher"))
        }
        LauncherType::CurseForgeApp => {
            let candidates = [
                home.join(".curseforge"),
                PathBuf::from("/opt/curseforge"),
            ];
            candidates
                .into_iter()
                .find(|p| p.exists())
                .unwrap_or(home.join(".curseforge"))
        }
        LauncherType::ATLauncher => home.join(".atlauncher"),
        LauncherType::GDLauncher => home.join(".gdlauncher"),
        LauncherType::Vanilla => home.join(".minecraft"),
    }
}

/// Scan MultiMC / Prism Launcher instances.
async fn scan_multimc(base: &std::path::Path) -> Result<Vec<ImportableInstance>, String> {
    let instances_dir = base.join("instances");
    if !instances_dir.exists() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    let mut entries = tokio::fs::read_dir(&instances_dir)
        .await
        .map_err(|e| e.to_string())?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let instance_cfg = entry.path().join("instance.cfg");
        if !instance_cfg.exists() {
            continue;
        }

        let content = tokio::fs::read_to_string(&instance_cfg)
            .await
            .unwrap_or_default();

        let mut name = entry.file_name().to_string_lossy().to_string();
        let mut game_version = String::new();
        let mut loader = "vanilla".to_string();
        let mut loader_version = None;

        for line in content.lines() {
            if let Some(val) = line.strip_prefix("name=") {
                name = val.to_string();
            }
            if let Some(val) = line.strip_prefix("IntendedVersion=") {
                game_version = val.to_string();
            }
            if line.starts_with("ForgeVersion=") && !line.contains("0") {
                loader = "forge".to_string();
                loader_version = line.strip_prefix("ForgeVersion=").map(|s| s.to_string());
            }
            if line.contains("net.fabricmc") {
                loader = "fabric".to_string();
            }
        }

        if !game_version.is_empty() {
            results.push(ImportableInstance {
                name,
                game_version,
                loader,
                loader_version,
                source_path: entry.path().to_string_lossy().to_string(),
                source_launcher: "MultiMC".to_string(),
                icon: None,
            });
        }
    }

    Ok(results)
}

/// Scan CurseForge App instances.
async fn scan_curseforge_app(base: &std::path::Path) -> Result<Vec<ImportableInstance>, String> {
    let profiles_dir = base.join("minecraft").join("Instances");
    if !profiles_dir.exists() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    let mut entries = tokio::fs::read_dir(&profiles_dir)
        .await
        .map_err(|e| e.to_string())?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let manifest = entry.path().join("manifest.json");
        if !manifest.exists() {
            continue;
        }

        let content = tokio::fs::read_to_string(&manifest)
            .await
            .unwrap_or_default();

        let manifest: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

        let name = entry.file_name().to_string_lossy().to_string();
        let game_version = manifest["minecraft"]["version"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut loader = "vanilla".to_string();
        let mut loader_version = None;

        if let Some(modloaders) = manifest["minecraft"]["modloaders"].as_array() {
            if let Some(first) = modloaders.first() {
                if let Some(id) = first["id"].as_str() {
                    if id.starts_with("forge-") {
                        loader = "forge".to_string();
                        loader_version = Some(id.strip_prefix("forge-").unwrap_or(id).to_string());
                    } else if id.starts_with("fabric-") {
                        loader = "fabric".to_string();
                        loader_version =
                            Some(id.strip_prefix("fabric-").unwrap_or(id).to_string());
                    }
                }
            }
        }

        if !game_version.is_empty() {
            results.push(ImportableInstance {
                name,
                game_version,
                loader,
                loader_version,
                source_path: entry.path().to_string_lossy().to_string(),
                source_launcher: "CurseForge".to_string(),
                icon: None,
            });
        }
    }

    Ok(results)
}

/// Scan ATLauncher instances.
async fn scan_atlauncher(base: &std::path::Path) -> Result<Vec<ImportableInstance>, String> {
    let instances_dir = base.join("instances");
    if !instances_dir.exists() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    let mut entries = tokio::fs::read_dir(&instances_dir)
        .await
        .map_err(|e| e.to_string())?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let instance_json = entry.path().join("instance.json");
        if !instance_json.exists() {
            continue;
        }

        let content = tokio::fs::read_to_string(&instance_json)
            .await
            .unwrap_or_default();

        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

        let name = data["name"].as_str().unwrap_or("Unknown").to_string();
        let game_version = data["minecraftVersion"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut loader = "vanilla".to_string();
        let mut loader_version = None;

        if let Some(forge) = data["loaderVersion"].as_str() {
            if !forge.is_empty() {
                loader = "forge".to_string();
                loader_version = Some(forge.to_string());
            }
        }

        if !game_version.is_empty() {
            results.push(ImportableInstance {
                name,
                game_version,
                loader,
                loader_version,
                source_path: entry.path().to_string_lossy().to_string(),
                source_launcher: "ATLauncher".to_string(),
                icon: None,
            });
        }
    }

    Ok(results)
}

/// Scan GDLauncher instances.
async fn scan_gdlauncher(base: &std::path::Path) -> Result<Vec<ImportableInstance>, String> {
    let instances_dir = base.join("instances");
    if !instances_dir.exists() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    let mut entries = tokio::fs::read_dir(&instances_dir)
        .await
        .map_err(|e| e.to_string())?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let config = entry.path().join("config.json");
        if !config.exists() {
            continue;
        }

        let content = tokio::fs::read_to_string(&config)
            .await
            .unwrap_or_default();

        let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

        let name = data["name"]
            .as_str()
            .unwrap_or(&entry.file_name().to_string_lossy())
            .to_string();
        let game_version = data["version"].as_str().unwrap_or("").to_string();

        let mut loader = "vanilla".to_string();
        let mut loader_version = None;

        if let Some(modloader) = data["modloader"].as_str() {
            if modloader != "vanilla" {
                loader = modloader.to_string();
                loader_version = data["modloaderVersion"].as_str().map(|s| s.to_string());
            }
        }

        if !game_version.is_empty() {
            results.push(ImportableInstance {
                name,
                game_version,
                loader,
                loader_version,
                source_path: entry.path().to_string_lossy().to_string(),
                source_launcher: "GDLauncher".to_string(),
                icon: None,
            });
        }
    }

    Ok(results)
}

/// Scan vanilla Minecraft launcher instances (profiles).
async fn scan_vanilla(base: &std::path::Path) -> Result<Vec<ImportableInstance>, String> {
    let launcher_profiles = base.join("launcher_profiles.json");
    if !launcher_profiles.exists() {
        return Ok(vec![]);
    }

    let content = tokio::fs::read_to_string(&launcher_profiles)
        .await
        .unwrap_or_default();

    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();

    let mut results = Vec::new();

    if let Some(profiles) = data["profiles"].as_object() {
        for (id, profile) in profiles {
            let name = profile["name"].as_str().unwrap_or(id).to_string();
            let game_version = profile["lastVersionId"]
                .as_str()
                .unwrap_or("")
                .to_string();

            // Skip versions that look like "latest-snapshot" etc
            if game_version.starts_with("latest-") || game_version.is_empty() {
                continue;
            }

            let game_dir = profile["gameDir"]
                .as_str()
                .map(PathBuf::from)
                .unwrap_or_else(|| base.to_path_buf());

            results.push(ImportableInstance {
                name,
                game_version,
                loader: "vanilla".to_string(),
                loader_version: None,
                source_path: game_dir.to_string_lossy().to_string(),
                source_launcher: "Vanilla".to_string(),
                icon: None,
            });
        }
    }

    Ok(results)
}

/// Recursively copy a directory.
async fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dst).await?;

    let mut entries = tokio::fs::read_dir(src).await?;

    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            tokio::fs::copy(&src_path, &dst_path).await?;
        }
    }

    Ok(())
}
