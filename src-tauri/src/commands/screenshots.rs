use crate::error::AppError;
use crate::utils::paths::data_dir;
use serde::Serialize;
use std::fs;

#[derive(Serialize, Clone, Debug)]
pub struct ScreenshotInfo {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: String,
}

fn parse_date_from_filename(name: &str) -> String {
    // Minecraft format: YYYY-MM-DD_HH.MM.SS.png
    let base = name.trim_end_matches(".png").trim_end_matches(".jpg");
    if let Some((date_part, time_part)) = base.split_once('_') {
        let time_fixed = time_part.replace('.', ":");
        format!("{} {}", date_part, time_fixed)
    } else {
        name.to_string()
    }
}

#[tauri::command]
pub fn list_screenshots(instance_id: String) -> Result<Vec<ScreenshotInfo>, AppError> {
    let dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("screenshots");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut screenshots = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let lower = name.to_lowercase();
        if !lower.ends_with(".png") && !lower.ends_with(".jpg") && !lower.ends_with(".jpeg") {
            continue;
        }
        let meta = entry.metadata()?;
        let path = entry.path().to_string_lossy().to_string();
        screenshots.push(ScreenshotInfo {
            created_at: parse_date_from_filename(&name),
            filename: name,
            path,
            size_bytes: meta.len(),
        });
    }
    screenshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(screenshots)
}

#[tauri::command]
pub fn delete_screenshot(instance_id: String, filename: String) -> Result<(), AppError> {
    let path = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("screenshots")
        .join(&filename);
    if !path.exists() {
        return Err(AppError::Internal(format!("{}", "")));
    }
    fs::remove_file(&path).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn open_screenshots_folder(instance_id: String) -> Result<(), AppError> {
    let dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("screenshots");
    fs::create_dir_all(&dir).ok();
    opener::open(&dir).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn export_screenshot(
    instance_id: String,
    filename: String,
    dest_path: String,
) -> Result<(), AppError> {
    let src = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("screenshots")
        .join(&filename);
    if !src.exists() {
        return Err(AppError::Internal(format!("{}", "")));
    }
    fs::copy(&src, &dest_path)?;
    Ok(())
}
