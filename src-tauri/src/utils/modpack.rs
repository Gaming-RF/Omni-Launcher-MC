// Modpack import functionality
// Handles Modrinth .mrpack and CurseForge modpack ZIP formats.
//
// Modrinth .mrpack:
//   - A ZIP containing modrinth.index.json + overrides/ directory
//   - The index JSON lists all files to download (mods, resources)
//   - overrides/ contains config/resource files to copy
//
// CurseForge modpack:
//   - A ZIP containing manifest.json + overrides/ directory
//   - manifest.json lists mods with project/file IDs
//   - Download URLs require the CurseForge API

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;

// ── Modrinth .mrpack ──────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct MrpackIndex {
    #[serde(default, rename = "formatVersion", alias = "format_version")]
    pub format_version: u32,
    pub game: String,
    #[serde(rename = "versionId", alias = "version_id")]
    pub version_id: String,
    #[serde(default)]
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<MrpackFile>,
    #[serde(default)]
    pub dependencies: MrpackDependencies,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MrpackFile {
    pub path: String,
    pub hashes: std::collections::HashMap<String, String>,
    pub env: Option<MrpackEnv>,
    pub downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    pub file_size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MrpackEnv {
    pub client: String,
    pub server: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MrpackDependencies {
    pub minecraft: String,
    #[serde(rename = "fabric-loader")]
    pub fabric_loader: Option<String>,
    #[serde(rename = "quilt-loader")]
    pub quilt_loader: Option<String>,
    #[serde(rename = "forge")]
    pub forge: Option<String>,
    #[serde(rename = "neoforge")]
    pub neoforge: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackInfo {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub game_version: String,
    pub loader: String,
    pub loader_version: String,
    pub file_count: usize,
}

/// Parse a Modrinth .mrpack file and return its metadata.
pub fn parse_mrpack(path: &Path) -> Result<ModpackInfo> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Read modrinth.index.json
    let mut index_file = archive
        .by_name("modrinth.index.json")
        .context("Not a valid .mrpack file (missing modrinth.index.json)")?;
    let mut contents = String::new();
    index_file.read_to_string(&mut contents)?;
    let index: MrpackIndex = serde_json::from_str(&contents)?;

    // Determine loader and version from dependencies
    let (loader, loader_version) = if let Some(ref fv) = index.dependencies.fabric_loader {
        ("fabric".to_string(), fv.clone())
    } else if let Some(ref qv) = index.dependencies.quilt_loader {
        ("quilt".to_string(), qv.clone())
    } else if let Some(ref fv) = index.dependencies.forge {
        ("forge".to_string(), fv.clone())
    } else if let Some(ref nv) = index.dependencies.neoforge {
        ("neoforge".to_string(), nv.clone())
    } else {
        ("vanilla".to_string(), String::new())
    };

    Ok(ModpackInfo {
        name: index.name,
        version: index.version_id,
        summary: index.summary,
        game_version: index.dependencies.minecraft,
        loader,
        loader_version,
        file_count: index.files.len(),
    })
}

/// Install a Modrinth .mrpack into an instance directory.
/// Downloads all listed files and copies overrides.
pub async fn install_mrpack(
    path: &Path,
    instance_dir: &Path,
    http_client: &reqwest::Client,
) -> Result<ModpackInfo> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Read index
    let index: MrpackIndex = {
        let mut index_file = archive.by_name("modrinth.index.json")?;
        let mut contents = String::new();
        index_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    // Create instance directories
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;
    std::fs::create_dir_all(instance_dir.join("config"))?;

    // Download all listed files
    for file_entry in &index.files {
        let dest = crate::utils::validate::safe_join(instance_dir, &file_entry.path)
            .map_err(anyhow::Error::msg)?;
        if dest.exists() {
            continue;
        }

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Try each download URL
        let mut downloaded = false;
        for url in &file_entry.downloads {
            match http_client.get(url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let bytes = resp.bytes().await?;
                    std::fs::write(&dest, &bytes)?;
                    downloaded = true;
                    break;
                }
                _ => continue,
            }
        }

        if !downloaded {
            log::warn!("Failed to download: {}", file_entry.path);
        }
    }

    // Copy overrides directory
    let overrides_exist = archive.by_name("overrides/").is_ok();
    if overrides_exist {
        // Re-open archive since we consumed it
        let file2 = std::fs::File::open(path)?;
        let mut archive2 = zip::ZipArchive::new(file2)?;

        for i in 0..archive2.len() {
            let mut entry = archive2.by_index(i)?;
            let name = entry.name().to_string();

            if !name.starts_with("overrides/") {
                continue;
            }

            let relative = &name["overrides/".len()..];
            if relative.is_empty() {
                continue;
            }

            let dest = crate::utils::validate::safe_join(instance_dir, relative)
                .map_err(anyhow::Error::msg)?;
            if entry.is_dir() {
                std::fs::create_dir_all(&dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                std::fs::write(&dest, &buf)?;
            }
        }
    }

    // Return modpack info
    let (loader, loader_version) = if let Some(ref fv) = index.dependencies.fabric_loader {
        ("fabric".to_string(), fv.clone())
    } else if let Some(ref qv) = index.dependencies.quilt_loader {
        ("quilt".to_string(), qv.clone())
    } else if let Some(ref fv) = index.dependencies.forge {
        ("forge".to_string(), fv.clone())
    } else if let Some(ref nv) = index.dependencies.neoforge {
        ("neoforge".to_string(), nv.clone())
    } else {
        ("vanilla".to_string(), String::new())
    };

    Ok(ModpackInfo {
        name: index.name,
        version: index.version_id,
        summary: index.summary,
        game_version: index.dependencies.minecraft,
        loader,
        loader_version,
        file_count: index.files.len(),
    })
}

// ── CurseForge Modpack ────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CfManifest {
    #[serde(rename = "minecraft")]
    pub minecraft: CfMinecraft,
    #[serde(rename = "manifestType")]
    pub manifest_type: String,
    #[serde(rename = "manifestVersion")]
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    pub author: String,
    pub files: Vec<CfManifestFile>,
    pub overrides: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CfMinecraft {
    pub version: String,
    #[serde(rename = "modLoaders")]
    pub mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CfModLoader {
    pub id: String,
    pub primary: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CfManifestFile {
    #[serde(rename = "projectID")]
    pub project_id: i32,
    #[serde(rename = "fileID")]
    pub file_id: i32,
    pub required: bool,
}

/// Parse a CurseForge modpack ZIP and return its metadata.
pub fn parse_cf_modpack(path: &Path) -> Result<ModpackInfo> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut manifest_file = archive
        .by_name("manifest.json")
        .context("Not a valid CurseForge modpack (missing manifest.json)")?;
    let mut contents = String::new();
    manifest_file.read_to_string(&mut contents)?;
    let manifest: CfManifest = serde_json::from_str(&contents)?;

    let game_version = manifest.minecraft.version.clone();
    let primary_loader = manifest.minecraft.mod_loaders.iter().find(|m| m.primary);

    let (loader, loader_version) = if let Some(ml) = primary_loader {
        let parts: Vec<&str> = ml.id.split('-').collect();
        if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("unknown".to_string(), ml.id.clone())
        }
    } else {
        ("vanilla".to_string(), String::new())
    };

    Ok(ModpackInfo {
        name: manifest.name,
        version: manifest.version,
        summary: None,
        game_version,
        loader,
        loader_version,
        file_count: manifest.files.len(),
    })
}

/// Install a CurseForge modpack into an instance directory.
/// Copies overrides and returns the list of mod project/file IDs for download.
pub async fn install_cf_modpack(
    path: &Path,
    instance_dir: &Path,
) -> Result<(ModpackInfo, Vec<CfManifestFile>)> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut manifest_file = archive.by_name("manifest.json")?;
    let mut contents = String::new();
    manifest_file.read_to_string(&mut contents)?;
    let manifest: CfManifest = serde_json::from_str(&contents)?;

    // Copy overrides
    let overrides_dir = &manifest.overrides;
    let prefix = format!("{}/", overrides_dir);

    // Re-open for reading
    let file2 = std::fs::File::open(path)?;
    let mut archive2 = zip::ZipArchive::new(file2)?;

    for i in 0..archive2.len() {
        let mut entry = archive2.by_index(i)?;
        let name = entry.name().to_string();

        if !name.starts_with(&prefix) {
            continue;
        }

        let relative = &name[prefix.len()..];
        if relative.is_empty() {
            continue;
        }

        let dest = crate::utils::validate::safe_join(instance_dir, relative)
            .map_err(anyhow::Error::msg)?;
        if entry.is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            std::fs::write(&dest, &buf)?;
        }
    }

    let game_version = manifest.minecraft.version.clone();
    let primary_loader = manifest.minecraft.mod_loaders.iter().find(|m| m.primary);
    let (loader, loader_version) = if let Some(ml) = primary_loader {
        let parts: Vec<&str> = ml.id.split('-').collect();
        if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("unknown".to_string(), ml.id.clone())
        }
    } else {
        ("vanilla".to_string(), String::new())
    };

    let info = ModpackInfo {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        summary: None,
        game_version,
        loader,
        loader_version,
        file_count: manifest.files.len(),
    };

    Ok((info, manifest.files))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_mrpack_accepts_camel_case_metadata() {
        let dir = std::env::temp_dir().join(format!("olmc-mrpack-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let pack_path = dir.join("pack.mrpack");
        {
            let file = std::fs::File::create(&pack_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            zip.add_directory("overrides", zip::write::SimpleFileOptions::default())
                .unwrap();
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("modrinth.index.json", options).unwrap();
            zip.write_all(
                br#"{
                    "formatVersion": 1,
                    "game": "minecraft",
                    "versionId": "1.2.3",
                    "name": "Demo",
                    "files": [],
                    "dependencies": { "minecraft": "1.20.1" }
                }"#,
            )
            .unwrap();
            zip.start_file("overrides/config/demo.txt", options)
                .unwrap();
            zip.write_all(b"demo=true").unwrap();
            zip.finish().unwrap();
        }

        let info = parse_mrpack(&pack_path).expect("camelCase metadata should parse");
        assert_eq!(info.version, "1.2.3");
        assert_eq!(info.game_version, "1.20.1");
        let target = dir.join("instance");
        let client = reqwest::Client::new();
        let installed = futures::executor::block_on(install_mrpack(&pack_path, &target, &client))
            .expect("camelCase archive should install");
        assert_eq!(installed.version, info.version);
        assert_eq!(
            std::fs::read(target.join("config/demo.txt")).unwrap(),
            b"demo=true"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_mrpack_preserves_legacy_metadata_support() {
        let index: MrpackIndex = serde_json::from_value(serde_json::json!({
            "format_version": 1,
            "version_id": "legacy",
            "game": "minecraft",
            "files": [],
            "dependencies": { "minecraft": "1.20.1" }
        }))
        .unwrap();
        assert_eq!(index.format_version, 1);
        assert_eq!(index.version_id, "legacy");
        let serialized = serde_json::to_value(index).unwrap();
        assert_eq!(serialized["formatVersion"], 1);
        assert_eq!(serialized["versionId"], "legacy");
        assert!(serialized.get("version_id").is_none());
    }

    #[test]
    fn test_cf_manifest_rejects_path_traversal_override() {
        let dir = std::env::temp_dir().join(format!("olmc-cf-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let pack_path = dir.join("pack.zip");
        {
            let file = std::fs::File::create(&pack_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("overrides/../evil.txt", options).unwrap();
            zip.write_all(b"payload").unwrap();
            zip.start_file("manifest.json", options).unwrap();
            zip.write_all(
                br#"{
                    "minecraft": { "version": "1.20.1", "modLoaders": [] },
                    "manifestType": "modpack",
                    "manifestVersion": 1,
                    "name": "Evil",
                    "version": "1.0",
                    "author": "x",
                    "files": [],
                    "overrides": "overrides"
                }"#,
            )
            .unwrap();
            zip.finish().unwrap();
        }

        let target = dir.join("instance");
        std::fs::create_dir_all(&target).unwrap();
        let result = futures::executor::block_on(install_cf_modpack(&pack_path, &target));
        assert!(result.is_err(), "path traversal in overrides must fail");
        assert!(!dir.join("evil.txt").exists(), "no file outside instance");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
