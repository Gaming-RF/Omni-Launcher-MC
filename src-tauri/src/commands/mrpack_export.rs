//! Modpack export as .mrpack (Modrinth format).
//!
//! A `.mrpack` is a ZIP archive containing:
//! - `modrinth.index.json` — manifest with mod list, hashes, and download URLs
//! - `overrides/` — config files, resource packs, shader packs, saves, etc.
//!
//! Modrinth-sourced mods with valid hashes go into the `files` array in the index.
//! CurseForge mods (or mods without computable hashes) are placed in `overrides/mods/`.

use crate::db;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use tauri::State;

// ── Result type ────────────────────────────────────────────────

#[derive(Serialize)]
pub struct MrpackExportResult {
    pub path: String,
    pub file_count: usize,
    pub total_size_bytes: u64,
}

// ── Mrpack JSON schema ────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct MrpackIndex {
    format_version: i32, // must be 1
    game: String,        // "minecraft"
    version_id: String,  // semantic version of the modpack
    name: String,
    summary: Option<String>,
    dependencies: serde_json::Value,
    files: Vec<MrpackFile>,
}

#[derive(Serialize, Deserialize)]
struct MrpackFile {
    path: String, // relative path inside the mrpack (e.g. "mods/sodium-0.5.jar")
    hashes: MrpackHashes,
    env: Option<MrpackEnv>,
    downloads: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct MrpackHashes {
    sha1: String,
    sha512: String,
}

#[derive(Serialize, Deserialize)]
struct MrpackEnv {
    client: String, // "required" | "optional" | "unsupported"
    server: String,
}

// ── Helpers ────────────────────────────────────────────────────

/// Compute SHA-1 and SHA-512 hex digests of a file.
fn compute_hashes(path: &std::path::Path) -> anyhow::Result<(String, String)> {
    // sha1::Digest re-exports digest::Digest, usable by both Sha1 and Sha512.
    use sha1::Digest;
    use sha1::Sha1;
    use sha2::Sha512;

    let mut file = std::fs::File::open(path)?;
    let mut sha1 = Sha1::new();
    let mut sha512 = Sha512::new();
    let mut buf = [0u8; 8192];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sha1.update(&buf[..n]);
        sha512.update(&buf[..n]);
    }

    Ok((hex::encode(sha1.finalize()), hex::encode(sha512.finalize())))
}

/// Map a loader name to the Modrinth mrpack dependency key.
fn loader_dep_key(loader: &str) -> Option<&'static str> {
    match loader.to_lowercase().as_str() {
        "fabric" => Some("fabric-loader"),
        "forge" => Some("forge"),
        "neoforge" => Some("neoforge"),
        "quilt" => Some("quilt-loader"),
        _ => None,
    }
}

/// Recursively add all files from `dir` into the zip under `prefix/`.
fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &std::path::Path,
    prefix: &str,
    options: zip::write::SimpleFileOptions,
    total_size: &mut u64,
    file_count: &mut usize,
) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let zip_path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };

        if entry.file_type()?.is_dir() {
            add_dir_to_zip(
                zip,
                &entry.path(),
                &zip_path,
                options,
                total_size,
                file_count,
            )?;
        } else {
            let mut f = std::fs::File::open(entry.path())?;
            let meta = f.metadata()?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.start_file(&zip_path, options)?;
            zip.write_all(&buf)?;
            *total_size += meta.len();
            *file_count += 1;
        }
    }
    Ok(())
}

// ── Core export logic ─────────────────────────────────────────

/// Build a `.mrpack` ZIP at `dest_path` from instance data and installed mods.
fn build_mrpack_zip(
    instance: &db::instances::GameInstance,
    mods: &[db::mods::InstalledMod],
    dest_path: &std::path::Path,
    include_configs: bool,
    include_resourcepacks: bool,
    include_saves: bool,
) -> Result<MrpackExportResult, String> {
    let instance_dir = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance.id);
    let mods_dir = instance_dir.join("mods");

    // ── Categorise mods ────────────────────────────────────────

    let mut mrpack_files: Vec<MrpackFile> = Vec::new();
    // (filename, absolute path) — will be copied into overrides/mods/
    let mut override_mods: Vec<(String, std::path::PathBuf)> = Vec::new();

    for m in mods.iter().filter(|m| m.enabled) {
        let file_path = mods_dir.join(&m.file_name);
        if !file_path.exists() {
            continue;
        }

        if m.source == "modrinth" {
            match compute_hashes(&file_path) {
                Ok((sha1, sha512)) => {
                    mrpack_files.push(MrpackFile {
                        path: format!("mods/{}", m.file_name),
                        hashes: MrpackHashes { sha1, sha512 },
                        env: Some(MrpackEnv {
                            client: "required".to_string(),
                            server: "required".to_string(),
                        }),
                        downloads: vec![format!(
                            "cdn.modrinth.com/data/{}/versions/{}",
                            m.mod_id, m.version
                        )],
                    });
                }
                Err(e) => {
                    log::warn!(
                        "Cannot hash mod '{}' ({}), falling back to override: {e}",
                        m.name,
                        m.file_name,
                    );
                    override_mods.push((m.file_name.clone(), file_path));
                }
            }
        } else {
            // CurseForge or unknown source — put in overrides
            override_mods.push((m.file_name.clone(), file_path));
        }
    }

    // ── Build dependencies ─────────────────────────────────────

    let mut deps = serde_json::Map::new();
    deps.insert(
        "minecraft".into(),
        serde_json::Value::String(instance.game_version.clone()),
    );
    if let Some(ref lv) = instance.loader_version {
        if let Some(key) = loader_dep_key(&instance.loader) {
            deps.insert(key.into(), serde_json::Value::String(lv.clone()));
        }
    }

    let index = MrpackIndex {
        format_version: 1,
        game: "minecraft".into(),
        version_id: "1.0.0".into(),
        name: instance.name.clone(),
        summary: instance.notes.clone(),
        dependencies: serde_json::Value::Object(deps),
        files: mrpack_files,
    };

    // ── Create ZIP ─────────────────────────────────────────────

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create output directory: {e}"))?;
    }

    let file = std::fs::File::create(dest_path).map_err(|e| format!("Cannot create file: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut total_size: u64 = 0;
    let mut file_count: usize = 0;

    // 1. modrinth.index.json
    let json = serde_json::to_string_pretty(&index)
        .map_err(|e| format!("JSON serialisation error: {e}"))?;
    zip.start_file("modrinth.index.json", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    total_size += json.len() as u64;
    file_count += 1;

    // 2. Override mod files (CurseForge mods, mods without hashes)
    for (fname, path) in &override_mods {
        let mut f =
            std::fs::File::open(path).map_err(|e| format!("Failed to read mod {fname}: {e}"))?;
        let meta = f.metadata().map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        zip.start_file(format!("overrides/mods/{fname}"), opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(&buf).map_err(|e| e.to_string())?;
        total_size += meta.len();
        file_count += 1;
    }

    // 3. Override directories (config, resourcepacks, saves, etc.)
    let override_dirs: Vec<(&str, bool)> = vec![
        ("config", include_configs),
        ("resourcepacks", include_resourcepacks),
        ("shaderpacks", include_resourcepacks),
        ("saves", include_saves),
    ];

    for (dir_name, include) in override_dirs {
        if include {
            let src = instance_dir.join(dir_name);
            if src.is_dir() {
                add_dir_to_zip(
                    &mut zip,
                    &src,
                    &format!("overrides/{dir_name}"),
                    opts,
                    &mut total_size,
                    &mut file_count,
                )
                .map_err(|e| format!("Adding {dir_name}/: {e}"))?;
            }
        }
    }

    zip.finish().map_err(|e| e.to_string())?;

    // Use actual file size for accuracy.
    let final_size = std::fs::metadata(dest_path)
        .map(|m| m.len())
        .unwrap_or(total_size);

    log::info!(
        "Exported mrpack '{}': {file_count} entries, {final_size} bytes -> {}",
        instance.name,
        dest_path.display(),
    );

    Ok(MrpackExportResult {
        path: dest_path.to_string_lossy().to_string(),
        file_count,
        total_size_bytes: final_size,
    })
}

// ── Tauri commands ─────────────────────────────────────────────

/// Export an instance as `.mrpack` to the default exports directory.
#[tauri::command]
pub async fn export_mrpack(
    state: State<'_, AppState>,
    instance_id: String,
    include_configs: bool,
    include_resourcepacks: bool,
    include_saves: bool,
) -> Result<MrpackExportResult, String> {
    let (instance, mods) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let instance = db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?;
        let mods = db::mods::get_instance_mods(&db, &instance_id).map_err(|e| e.to_string())?;
        (instance, mods)
    };

    let exports_dir = crate::utils::paths::data_dir().join("exports");
    std::fs::create_dir_all(&exports_dir)
        .map_err(|e| format!("Cannot create exports directory: {e}"))?;

    let safe_name: String = instance
        .name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let dest = exports_dir.join(format!("{safe_name}.mrpack"));

    build_mrpack_zip(
        &instance,
        &mods,
        &dest,
        include_configs,
        include_resourcepacks,
        include_saves,
    )
}

/// Export an instance as `.mrpack` to a user-chosen path (used with save dialog).
#[tauri::command]
pub async fn export_mrpack_to_path(
    state: State<'_, AppState>,
    instance_id: String,
    dest_path: String,
    include_configs: bool,
    include_resourcepacks: bool,
    include_saves: bool,
) -> Result<MrpackExportResult, String> {
    let (instance, mods) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let instance = db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?;
        let mods = db::mods::get_instance_mods(&db, &instance_id).map_err(|e| e.to_string())?;
        (instance, mods)
    };

    let dest = std::path::PathBuf::from(&dest_path);

    build_mrpack_zip(
        &instance,
        &mods,
        &dest,
        include_configs,
        include_resourcepacks,
        include_saves,
    )
}
