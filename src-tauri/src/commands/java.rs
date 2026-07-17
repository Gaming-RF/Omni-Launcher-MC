// Tauri commands for Java management, mod loader installation, and modpack installation.
// These bridge the backend utils to the frontend via IPC.

use crate::utils::{java, loaders, modpack};
use serde::Serialize;

// ── Java Commands ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct JavaInfo {
    pub id: String,
    pub path: String,
    pub major_version: u32,
    pub arch: String,
    pub vendor: String,
    pub is_auto_downloaded: bool,
}

impl From<java::JavaInstallation> for JavaInfo {
    fn from(j: java::JavaInstallation) -> Self {
        Self {
            id: j.id,
            path: j.path.to_string_lossy().to_string(),
            major_version: j.major_version,
            arch: j.arch,
            vendor: j.vendor,
            is_auto_downloaded: j.is_auto_downloaded,
        }
    }
}

/// Detect all Java installations on the system.
#[tauri::command]
pub async fn detect_java_installations() -> Result<Vec<JavaInfo>, String> {
    let installations = java::detect_all_javas().await;
    Ok(installations.into_iter().map(JavaInfo::from).collect())
}

/// Auto-download a Java runtime for a given major version.
#[tauri::command]
pub async fn auto_download_java(major_version: u32) -> Result<JavaInfo, String> {
    let dest_dir = crate::utils::paths::data_dir().join("java");
    let installation = java::auto_download_java(major_version, &dest_dir)
        .await
        .map_err(|e| e.to_string())?;
    Ok(JavaInfo::from(installation))
}

/// Find the best Java for a specific Minecraft version.
#[tauri::command]
pub async fn find_java_for_mc(mc_version: String) -> Result<Option<JavaInfo>, String> {
    let installations = java::detect_all_javas().await;
    Ok(java::best_java_for_mc(&installations, &mc_version).map(|j| JavaInfo::from(j.clone())))
}

// ── Mod Loader Commands ───────────────────────────────────────

#[derive(Serialize)]
pub struct LoaderVersionInfo {
    pub loader: String,
    pub version: String,
    pub stable: bool,
    pub game_versions: Vec<String>,
}

/// Fetch available loader versions for a game version.
#[tauri::command]
pub async fn fetch_loader_versions(
    loader: String,
    game_version: String,
) -> Result<Vec<LoaderVersionInfo>, String> {
    let loader_type = loaders::ModLoader::from_str(&loader);

    let versions = match loader_type {
        loaders::ModLoader::Fabric => loaders::fetch_fabric_loaders(&game_version)
            .await
            .map_err(|e| e.to_string())?,
        loaders::ModLoader::Forge => loaders::fetch_forge_versions()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|v| v.game_versions.contains(&game_version))
            .collect(),
        loaders::ModLoader::NeoForge => loaders::fetch_neoforge_versions(&game_version)
            .await
            .map_err(|e| e.to_string())?,
        loaders::ModLoader::Quilt => loaders::fetch_quilt_loaders(&game_version)
            .await
            .map_err(|e| e.to_string())?,
        loaders::ModLoader::Vanilla => vec![],
    };

    Ok(versions
        .into_iter()
        .map(|v| LoaderVersionInfo {
            loader: v.loader.as_str().to_string(),
            version: v.version,
            stable: v.stable,
            game_versions: v.game_versions,
        })
        .collect())
}

/// Install a mod loader for an instance.
#[tauri::command]
pub async fn install_mod_loader(
    loader: String,
    game_version: String,
    loader_version: String,
) -> Result<String, String> {
    let loader_type = loaders::ModLoader::from_str(&loader);
    let versions_dir = crate::utils::paths::versions_dir();
    let java_path = crate::utils::launcher::find_java(None).ok();

    let profile_path = loaders::install_loader(
        &loader_type,
        &game_version,
        &loader_version,
        &versions_dir,
        java_path.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(profile_path.to_string_lossy().to_string())
}

// ── Modpack Commands ──────────────────────────────────────────

#[derive(Serialize)]
pub struct ModpackInfo {
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: String,
    pub file_count: usize,
    pub summary: Option<String>,
}

/// Parse a Modrinth .mrpack file and return its info.
#[tauri::command]
pub async fn parse_mrpack_file(file_path: String) -> Result<ModpackInfo, String> {
    let path = std::path::PathBuf::from(&file_path);
    let temp_dir = std::env::temp_dir().join("omnilauncher-parse");
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let parsed = modpack::parse_mrpack(&path, &temp_dir).map_err(|e| e.to_string())?;

    Ok(ModpackInfo {
        name: parsed.index.name,
        game_version: parsed.game_version,
        loader: parsed.loader,
        loader_version: parsed.loader_version,
        file_count: parsed.downloads.len(),
        summary: parsed.index.summary,
    })
}

/// Parse a CurseForge modpack ZIP and return its info.
#[tauri::command]
pub async fn parse_cf_modpack_file(file_path: String) -> Result<ModpackInfo, String> {
    let path = std::path::PathBuf::from(&file_path);
    let parsed = modpack::parse_cf_modpack(&path).map_err(|e| e.to_string())?;

    Ok(ModpackInfo {
        name: parsed.manifest.name,
        game_version: parsed.game_version,
        loader: parsed.loader,
        loader_version: parsed.loader_version,
        file_count: parsed.files_to_resolve.len(),
        summary: None,
    })
}
