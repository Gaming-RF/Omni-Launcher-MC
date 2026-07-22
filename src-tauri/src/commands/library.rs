use crate::error::AppError;
use crate::utils::paths::data_dir;
use serde::Serialize;
use sha1::{Digest, Sha1};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Clone, Debug)]
pub struct LibraryItem {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub item_type: String,
    pub source: Option<String>,
    pub file_size: u64,
    pub added_at: String,
    pub used_by: Vec<String>,
}

fn library_dir() -> PathBuf {
    data_dir().join("library")
}

fn scan_type(item_type: &str) -> Vec<LibraryItem> {
    let dir = library_dir().join(item_type);
    if !dir.exists() {
        return Vec::new();
    }
    let mut items = Vec::new();
    for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
        let item_dir = entry.path();
        if !item_dir.is_dir() {
            continue;
        }
        let meta_path = item_dir.join("meta.json");
        let meta: serde_json::Value = if meta_path.exists() {
            fs::read_to_string(&meta_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            serde_json::Value::Null
        };
        // Find the actual file
        let file_entry = fs::read_dir(&item_dir)
            .into_iter()
            .flatten()
            .flatten()
            .find(|e| e.file_name() != "meta.json");
        let (file_name, file_size) = if let Some(f) = file_entry {
            let size = f.metadata().map(|m| m.len()).unwrap_or(0);
            (f.file_name().to_string_lossy().to_string(), size)
        } else {
            continue;
        };
        // Find instances that link to this
        let used_by = find_usage(item_type, &entry.file_name().to_string_lossy());
        items.push(LibraryItem {
            id: entry.file_name().to_string_lossy().to_string(),
            name: meta
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(&file_name)
                .to_string(),
            file_name,
            item_type: item_type.to_string(),
            source: meta
                .get("source")
                .and_then(|v| v.as_str())
                .map(String::from),
            file_size,
            added_at: meta
                .get("added_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            used_by,
        });
    }
    items
}

fn find_usage(item_type: &str, hash_dir: &str) -> Vec<String> {
    let instances_dir = data_dir().join("instances");
    let mut users = Vec::new();
    if !instances_dir.exists() {
        return users;
    }
    let sub = match item_type {
        "mods" => "mods",
        "resourcepacks" => "resourcepacks",
        "shaderpacks" => "shaderpacks",
        _ => return users,
    };
    let lib_path = library_dir().join(item_type).join(hash_dir);
    let Ok(canonical_lib) = fs::canonicalize(&lib_path) else {
        return users;
    };
    for entry in fs::read_dir(&instances_dir).into_iter().flatten().flatten() {
        let target = entry.path().join(".minecraft").join(sub);
        if !target.exists() {
            continue;
        }
        for f in fs::read_dir(&target).into_iter().flatten().flatten() {
            if f.path().is_symlink() {
                if let Ok(link_target) = fs::canonicalize(f.path()) {
                    if link_target.starts_with(&canonical_lib) {
                        users.push(entry.file_name().to_string_lossy().to_string());
                        break;
                    }
                }
            }
        }
    }
    users
}

#[tauri::command]
pub fn list_library_items(item_type: Option<String>) -> Result<Vec<LibraryItem>, AppError> {
    let types: Vec<&str> = if let Some(ref t) = item_type {
        vec![t.as_str()]
    } else {
        vec!["mods", "resourcepacks", "shaderpacks"]
    };
    let mut all = Vec::new();
    for t in types {
        all.extend(scan_type(t));
    }
    all.sort_by(|a, b| b.added_at.cmp(&a.added_at));
    Ok(all)
}

#[tauri::command]
pub fn import_to_library(instance_id: String, file_name: String) -> Result<LibraryItem, AppError> {
    let instances = data_dir().join("instances").join(&instance_id);
    // Try mods/, resourcepacks/, shaderpacks/
    let (src_path, item_type) = [
        ("mods", "mods"),
        ("resourcepacks", "resourcepacks"),
        ("shaderpacks", "shaderpacks"),
    ]
    .iter()
    .find_map(|(sub, t)| {
        let p = instances.join(".minecraft").join(sub).join(&file_name);
        if p.exists() {
            Some((p, *t))
        } else {
            None
        }
    })
    .ok_or_else(|| format!("File not found in instance: {}", file_name))?;

    // Compute hash
    let bytes = fs::read(&src_path)?;
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    let hash = format!("{:x}", hasher.finalize());

    // Create library entry
    let lib_dir = library_dir().join(item_type).join(&hash);
    fs::create_dir_all(&lib_dir)?;
    let dest = lib_dir.join(&file_name);
    if !dest.exists() {
        fs::copy(&src_path, &dest)?;
    }
    // Write meta
    let meta = serde_json::json!({
        "name": file_name.trim_end_matches(".jar").trim_end_matches(".zip"),
        "added_at": chrono::Utc::now().to_rfc3339(),
        "source": "manual",
    });
    let meta_json =
        serde_json::to_string_pretty(&meta).map_err(|e| format!("Failed to serialize metadata: {e}"))?;
    fs::write(lib_dir.join("meta.json"), meta_json).ok();

    // Replace original with symlink
    fs::remove_file(&src_path)?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(&dest, &src_path)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&dest, &src_path)?;

    let size = fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    Ok(LibraryItem {
        id: hash,
        name: file_name
            .trim_end_matches(".jar")
            .trim_end_matches(".zip")
            .to_string(),
        file_name,
        item_type: item_type.to_string(),
        source: Some("manual".into()),
        file_size: size,
        added_at: chrono::Utc::now().to_rfc3339(),
        used_by: vec![instance_id],
    })
}

#[tauri::command]
pub fn link_library_to_instance(
    library_id: String,
    instance_id: String,
    item_type: String,
) -> Result<(), AppError> {
    let lib_dir = library_dir().join(&item_type).join(&library_id);
    let file = fs::read_dir(&lib_dir)
        ?
        .flatten()
        .find(|e| e.file_name() != "meta.json")
        .ok_or("No file in library item")?;

    let sub = match item_type.as_str() {
        "mods" => "mods",
        "resourcepacks" => "resourcepacks",
        "shaderpacks" => "shaderpacks",
        _ => return Err(AppError::Validation("Invalid type".into())),
    };
    let target_dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join(".minecraft")
        .join(sub);
    fs::create_dir_all(&target_dir)?;
    let link = target_dir.join(file.file_name());
    if !link.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(file.path(), &link)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(file.path(), &link)?;
    }
    Ok(())
}

#[tauri::command]
pub fn unlink_library_from_instance(
    _library_id: String,
    instance_id: String,
    item_type: String,
    file_name: String,
) -> Result<(), AppError> {
    let sub = match item_type.as_str() {
        "mods" => "mods",
        "resourcepacks" => "resourcepacks",
        "shaderpacks" => "shaderpacks",
        _ => return Err(AppError::Validation("Invalid type".into())),
    };
    let link = data_dir()
        .join("instances")
        .join(&instance_id)
        .join(".minecraft")
        .join(sub)
        .join(&file_name);
    if link.is_symlink() {
        fs::remove_file(&link)?;
    }
    Ok(())
}

#[tauri::command]
pub fn cleanup_library() -> Result<(u64, u64), AppError> {
    let mut removed = 0u64;
    let mut freed = 0u64;
    for item_type in &["mods", "resourcepacks", "shaderpacks"] {
        let dir = library_dir().join(item_type);
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
            let usage = find_usage(item_type, &entry.file_name().to_string_lossy());
            if usage.is_empty() {
                let size: u64 = fs::read_dir(entry.path())
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum();
                fs::remove_dir_all(entry.path()).ok();
                removed += 1;
                freed += size;
            }
        }
    }
    Ok((removed, freed))
}
