use crate::utils::paths::data_dir;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Clone, Debug)]
pub struct ResourcePackInfo {
    pub name: String,
    pub filename: String,
    pub pack_type: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub size_bytes: u64,
}

fn scan_packs(dir: PathBuf, pack_type: &str) -> Vec<ResourcePackInfo> {
    let mut packs = Vec::new();
    if !dir.exists() {
        return packs;
    }
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return packs,
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let meta = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let is_pack = if path.is_dir() {
            pack_type == "shader"
        } else {
            name.ends_with(".zip")
        };
        if !is_pack {
            continue;
        }
        let enabled = !name.starts_with('_');
        let display_name = if enabled {
            name.trim_end_matches(".zip").to_string()
        } else {
            name.trim_start_matches('_')
                .trim_end_matches(".zip")
                .to_string()
        };
        let description = if name.ends_with(".zip") && path.is_file() {
            read_zip_description(&path).ok().flatten()
        } else {
            None
        };
        packs.push(ResourcePackInfo {
            name: display_name,
            filename: name,
            pack_type: pack_type.to_string(),
            description,
            enabled,
            size_bytes: meta.len(),
        });
    }
    packs.sort_by(|a, b| {
        b.enabled
            .cmp(&a.enabled)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    packs
}

fn read_zip_description(path: &PathBuf) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut mcmeta = match archive.by_name("pack.mcmeta") {
        Ok(f) => f,
        Err(_) => return Ok(None),
    };
    let mut contents = String::new();
    std::io::Read::read_to_string(&mut mcmeta, &mut contents)?;
    let json: serde_json::Value = serde_json::from_str(&contents)?;
    let desc = json
        .get("pack")
        .and_then(|p| p.get("description"))
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());
    Ok(desc)
}

fn toggle_pack_impl(dir: PathBuf, filename: &str, enabled: bool) -> Result<(), String> {
    let old_path = dir.join(filename);
    if !old_path.exists() {
        // Try finding with/without underscore prefix
        let alt = if filename.starts_with('_') {
            dir.join(filename.trim_start_matches('_'))
        } else {
            dir.join(format!("_{}", filename))
        };
        if alt.exists() {
            let target = if enabled {
                dir.join(
                    alt.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .trim_start_matches('_'),
                )
            } else {
                dir.join(format!("_{}", alt.file_name().unwrap().to_string_lossy()))
            };
            return fs::rename(&alt, &target).map_err(|e| e.to_string());
        }
        return Err(format!("File not found: {}", filename));
    }
    let target = if enabled {
        let stripped = filename.trim_start_matches('_');
        dir.join(stripped)
    } else {
        dir.join(format!("_{}", filename))
    };
    fs::rename(&old_path, &target).map_err(|e| e.to_string())
}

fn delete_pack_impl(dir: PathBuf, filename: &str) -> Result<(), String> {
    let path = dir.join(filename);
    if !path.exists() {
        return Err(format!("File not found: {}", filename));
    }
    if path.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| e.to_string())
    } else {
        fs::remove_file(&path).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn list_resource_packs(instance_id: String) -> Result<Vec<ResourcePackInfo>, String> {
    let dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("resourcepacks");
    Ok(scan_packs(dir, "resourcepack"))
}

#[tauri::command]
pub fn list_shaders(instance_id: String) -> Result<Vec<ResourcePackInfo>, String> {
    let dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("shaderpacks");
    Ok(scan_packs(dir, "shader"))
}

#[tauri::command]
pub fn toggle_resource_pack(
    instance_id: String,
    filename: String,
    enabled: bool,
) -> Result<(), String> {
    let dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("resourcepacks");
    toggle_pack_impl(dir, &filename, enabled)
}

#[tauri::command]
pub fn toggle_shader(instance_id: String, filename: String, enabled: bool) -> Result<(), String> {
    let dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("shaderpacks");
    toggle_pack_impl(dir, &filename, enabled)
}

#[tauri::command]
pub fn delete_resource_pack(instance_id: String, filename: String) -> Result<(), String> {
    let dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("resourcepacks");
    delete_pack_impl(dir, &filename)
}

#[tauri::command]
pub fn delete_shader(instance_id: String, filename: String) -> Result<(), String> {
    let dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("shaderpacks");
    delete_pack_impl(dir, &filename)
}

#[tauri::command]
pub fn open_resource_packs_folder(instance_id: String) -> Result<(), String> {
    let dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("resourcepacks");
    fs::create_dir_all(&dir).ok();
    opener::open(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_shaders_folder(instance_id: String) -> Result<(), String> {
    let dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join("shaderpacks");
    fs::create_dir_all(&dir).ok();
    opener::open(&dir).map_err(|e| e.to_string())
}
