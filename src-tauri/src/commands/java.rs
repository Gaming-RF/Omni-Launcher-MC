use crate::error::AppError;
use crate::utils::java;
use serde::Serialize;

#[derive(Serialize)]
pub struct JavaCheckResult {
    pub found: bool,
    pub path: Option<String>,
    pub major_version: u32,
    pub auto_downloaded: bool,
    pub error: Option<String>,
}

/// Check what Java version is required for a Minecraft version.
#[tauri::command]
pub fn get_required_java_version(mc_version: String) -> u32 {
    java::java_version_for_mc(&mc_version)
}

/// Find or auto-download the right Java for a Minecraft version.
#[tauri::command]
pub async fn ensure_java_for_mc(
    mc_version: String,
    custom_path: Option<String>,
) -> Result<JavaCheckResult, AppError> {
    let path = java::ensure_java(&mc_version, custom_path.as_deref())
        .await
        ?;

    let major = java::java_version_for_mc(&mc_version);
    let auto_downloaded = java::is_java_installed(major)
        .map(|p| p == path)
        .unwrap_or(false);

    Ok(JavaCheckResult {
        found: true,
        path: Some(path.to_string_lossy().to_string()),
        major_version: major,
        auto_downloaded,
        error: None,
    })
}

/// Download a specific Java version (explicit user action).
#[tauri::command]
pub async fn download_java_version(java_major: u32) -> Result<String, AppError> {
    let path = java::download_java(java_major)
        .await
        ?;
    Ok(path.to_string_lossy().to_string())
}
