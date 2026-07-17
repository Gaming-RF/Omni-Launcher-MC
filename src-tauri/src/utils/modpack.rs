// Modpack parser — .mrpack (Modrinth) and CurseForge modpack extraction + dependency resolution.
//
// Architecture:
// - Modrinth: Download .mrpack (ZIP) → extract modrinth.index.json → download listed files → copy overrides
// - CurseForge: Download modpack ZIP → extract manifest.json → resolve files via API → download → copy overrides
// - Dependency resolution: DAG of mod dependencies, topological sort for download order
// - All downloads go through the DownloadEngine (utils/download.rs)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ── Modrinth .mrpack ──────────────────────────────────────────

/// Modrinth modpack index (modrinth.index.json inside .mrpack).
#[derive(Debug, Deserialize)]
pub struct MrpackIndex {
    pub format_version: i32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<MrpackFile>,
    pub dependencies: HashMap<String, String>,
}

/// A file entry in the modpack index.
#[derive(Debug, Deserialize)]
pub struct MrpackFile {
    pub path: String,
    pub hashes: HashMap<String, String>,
    pub env: Option<MrpackEnv>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

/// Environment compatibility for a file.
#[derive(Debug, Deserialize)]
pub struct MrpackEnv {
    pub client: Option<String>,
    pub server: Option<String>,
}

/// Result of parsing a .mrpack file.
#[derive(Debug)]
pub struct ParsedMrpack {
    pub index: MrpackIndex,
    /// Files that need to be downloaded (not in overrides).
    pub downloads: Vec<MrpackDownloadTask>,
    /// Files from the overrides/ directory that should be copied.
    pub overrides: Vec<(PathBuf, PathBuf)>,
    /// Game version from dependencies.
    pub game_version: String,
    /// Loader from dependencies.
    pub loader: String,
    /// Loader version from dependencies.
    pub loader_version: String,
}

/// A download task extracted from .mrpack.
#[derive(Debug)]
pub struct MrpackDownloadTask {
    pub url: String,
    pub dest: PathBuf,
    pub sha512: String,
    pub size: u64,
    pub path: String,
}

/// Parse a .mrpack file (ZIP archive).
pub fn parse_mrpack(
    mrpack_path: &Path,
    instance_dir: &Path,
) -> Result<ParsedMrpack> {
    let file = std::fs::File::open(mrpack_path)
        .with_context(|| format!("Failed to open .mrpack: {}", mrpack_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .context("Failed to read .mrpack as ZIP")?;

    // 1. Extract and parse modrinth.index.json
    let index: MrpackIndex = {
        let mut index_file = archive
            .by_name("modrinth.index.json")
            .context("modrinth.index.json not found in .mrpack")?;
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut index_file, &mut contents)?;
        serde_json::from_str(&contents).context("Failed to parse modrinth.index.json")?
    };

    // 2. Determine game version and loader
    let game_version = index
        .dependencies
        .get("minecraft")
        .cloned()
        .unwrap_or_default();

    let (loader, loader_version) = if let Some(fv) = index.dependencies.get("fabric-loader") {
        ("fabric".to_string(), fv.clone())
    } else if let Some(fv) = index.dependencies.get("forge") {
        ("forge".to_string(), fv.clone())
    } else if let Some(fv) = index.dependencies.get("neoforge") {
        ("neoforge".to_string(), fv.clone())
    } else if let Some(fv) = index.dependencies.get("quilt-loader") {
        ("quilt".to_string(), fv.clone())
    } else {
        ("vanilla".to_string(), String::new())
    };

    // 3. Build download tasks from file list
    let game_dir = instance_dir.join(".minecraft");
    let mut downloads = Vec::new();

    for entry in &index.files {
        // Skip client-incompatible files
        if let Some(env) = &entry.env {
            if env.client.as_deref() == Some("unsupported") {
                continue;
            }
        }

        let dest = game_dir.join(&entry.path);
        let sha512 = entry
            .hashes
            .get("sha512")
            .cloned()
            .unwrap_or_default();
        let url = entry.downloads.first().cloned().unwrap_or_default();

        downloads.push(MrpackDownloadTask {
            url,
            dest,
            sha512,
            size: entry.file_size,
            path: entry.path.clone(),
        });
    }

    // 4. Extract overrides
    let mut overrides = Vec::new();
    let mut archive = zip::ZipArchive::new(
        std::fs::File::open(mrpack_path)
            .with_context(|| format!("Failed to reopen .mrpack: {}", mrpack_path.display()))?,
    )
    .context("Failed to reopen .mrpack")?;

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();

        // Only process "overrides/" directory
        if !entry_name.starts_with("overrides/") {
            continue;
        }

        let relative = entry_name
            .strip_prefix("overrides/")
            .unwrap_or(&entry_name);

        if relative.is_empty() {
            continue;
        }

        let dest = game_dir.join(relative);
        overrides.push((PathBuf::from(&entry_name), dest));
    }

    Ok(ParsedMrpack {
        index,
        downloads,
        overrides,
        game_version,
        loader,
        loader_version,
    })
}

/// Extract override files from .mrpack to the instance's game directory.
pub fn extract_mrpack_overrides(mrpack_path: &Path, instance_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(mrpack_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let game_dir = instance_dir.join(".minecraft");
    let override_dirs = ["overrides/", "client-overrides/"];

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        let relative = override_dirs.iter().find_map(|prefix| {
            name.strip_prefix(prefix).filter(|r| !r.is_empty())
        });

        if let Some(relative) = relative {
            let dest = game_dir.join(relative);

            if entry.is_dir() {
                std::fs::create_dir_all(&dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out = std::fs::File::create(&dest)?;
                std::io::copy(&mut entry, &mut out)?;
            }
        }
    }

    Ok(())
}

// ── CurseForge Modpack ────────────────────────────────────────

/// CurseForge modpack manifest.json structure.
#[derive(Debug, Deserialize)]
pub struct CfManifest {
    pub minecraft: CfMinecraft,
    pub manifest_type: String,
    pub manifest_version: i32,
    pub name: String,
    pub version: String,
    pub author: String,
    pub files: Vec<CfManifestFile>,
    pub overrides: String,
}

#[derive(Debug, Deserialize)]
pub struct CfMinecraft {
    pub version: String,
    pub mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Deserialize)]
pub struct CfModLoader {
    pub id: String,
    pub primary: bool,
}

/// A file entry in CurseForge manifest.
#[derive(Debug, Deserialize)]
pub struct CfManifestFile {
    pub project_id: i32,
    pub file_id: i32,
    pub required: bool,
}

/// Result of parsing a CurseForge modpack.
#[derive(Debug)]
pub struct ParsedCfModpack {
    pub manifest: CfManifest,
    pub game_version: String,
    pub loader: String,
    pub loader_version: String,
    /// Files that need API resolution (project_id + file_id pairs).
    pub files_to_resolve: Vec<(i32, i32)>,
    /// Override files to copy.
    pub overrides_dir: String,
}

/// Parse a CurseForge modpack ZIP.
pub fn parse_cf_modpack(zip_path: &Path) -> Result<ParsedCfModpack> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Extract manifest.json
    let manifest: CfManifest = {
        let mut manifest_file = archive
            .by_name("manifest.json")
            .context("manifest.json not found in CurseForge modpack")?;
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut manifest_file, &mut contents)?;
        serde_json::from_str(&contents).context("Failed to parse manifest.json")?
    };

    let game_version = manifest.minecraft.version.clone();

    // Parse loader info (e.g., "forge-47.2.0" or "neoforge-21.0.1")
    let (loader, loader_version) = if let Some(ml) = manifest.minecraft.mod_loaders.first() {
        let id = &ml.id;
        if let Some(v) = id.strip_prefix("forge-") {
            ("forge".to_string(), v.to_string())
        } else if let Some(v) = id.strip_prefix("neoforge-") {
            ("neoforge".to_string(), v.to_string())
        } else if let Some(v) = id.strip_prefix("fabric-") {
            ("fabric".to_string(), v.to_string())
        } else if let Some(v) = id.strip_prefix("quilt-") {
            ("quilt".to_string(), v.to_string())
        } else {
            ("unknown".to_string(), id.clone())
        }
    } else {
        ("vanilla".to_string(), String::new())
    };

    let files_to_resolve: Vec<_> = manifest
        .files
        .iter()
        .map(|f| (f.project_id, f.file_id))
        .collect();

    let overrides_dir = manifest.overrides.clone();

    Ok(ParsedCfModpack {
        manifest,
        game_version,
        loader,
        loader_version,
        files_to_resolve,
        overrides_dir,
    })
}

/// Extract override files from a CurseForge modpack.
pub fn extract_cf_overrides(zip_path: &Path, instance_dir: &Path, overrides_dir: &str) -> Result<()> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let game_dir = instance_dir.join(".minecraft");
    let prefix = format!("{}/", overrides_dir);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        if !name.starts_with(&prefix) {
            continue;
        }

        let relative = name.strip_prefix(&prefix).unwrap_or(&name);
        if relative.is_empty() {
            continue;
        }

        let dest = game_dir.join(relative);

        if entry.is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out = std::fs::File::create(&dest)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }

    Ok(())
}

// ── Dependency Resolution ─────────────────────────────────────

/// A resolved mod dependency with download info.
#[derive(Debug, Clone)]
pub struct ResolvedMod {
    pub project_id: String,
    pub source: String, // "modrinth" or "curseforge"
    pub name: String,
    pub version_id: String,
    pub download_url: String,
    pub filename: String,
    pub sha512: Option<String>,
    pub is_dependency: bool, // true if auto-resolved, false if user-selected
}

/// Resolve Modrinth mod dependencies.
/// For each required dependency, find the best matching version.
pub async fn resolve_modrinth_dependencies(
    project_id: &str,
    version_id: &str,
    game_version: &str,
    loader: &str,
) -> Result<Vec<ResolvedMod>> {
    let client = reqwest::Client::new();

    // Get the version to find its dependencies
    let url = format!("https://api.modrinth.com/v2/version/{}", version_id);
    let version: serde_json::Value = client
        .get(&url)
        .header("User-Agent", "OmniLauncherMC/0.1.0")
        .send()
        .await
        .context("Failed to fetch version for dependency resolution")?
        .json()
        .await
        .context("Failed to parse version")?;

    let dependencies = version["dependencies"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut resolved = Vec::new();
    let mut seen = HashSet::new();
    seen.insert(project_id.to_string());

    for dep in &dependencies {
        let dep_type = dep["dependency_type"].as_str().unwrap_or("required");
        if dep_type != "required" {
            continue;
        }

        let dep_project_id = dep["project_id"].as_str().unwrap_or_default();
        if dep_project_id.is_empty() || seen.contains(dep_project_id) {
            continue;
        }
        seen.insert(dep_project_id.to_string());

        // If specific version is pinned, use it
        if let Some(dep_version_id) = dep["version_id"].as_str() {
            if !dep_version_id.is_empty() {
                let ver_url = format!("https://api.modrinth.com/v2/version/{}", dep_version_id);
                if let Ok(ver_resp) = client
                    .get(&ver_url)
                    .header("User-Agent", "OmniLauncherMC/0.1.0")
                    .send()
                    .await
                {
                    if let Ok(ver) = ver_resp.json::<serde_json::Value>().await {
                        let name = ver["name"].as_str().unwrap_or("unknown").to_string();
                        let files = ver["files"].as_array().cloned().unwrap_or_default();
                        if let Some(primary) = files.iter().find(|f| f["primary"].as_bool().unwrap_or(true)) {
                            resolved.push(ResolvedMod {
                                project_id: dep_project_id.to_string(),
                                source: "modrinth".to_string(),
                                name,
                                version_id: dep_version_id.to_string(),
                                download_url: primary["url"].as_str().unwrap_or_default().to_string(),
                                filename: primary["filename"].as_str().unwrap_or_default().to_string(),
                                sha512: primary["hashes"]["sha512"].as_str().map(|s| s.to_string()),
                                is_dependency: true,
                            });
                        }
                    }
                }
                continue;
            }
        }

        // Otherwise, find the best matching version
        let versions_url = format!(
            "https://api.modrinth.com/v2/project/{}/version?loaders=[\"{}\"]&game_versions=[\"{}\"]",
            dep_project_id, loader, game_version
        );
        if let Ok(resp) = client
            .get(&versions_url)
            .header("User-Agent", "OmniLauncherMC/0.1.0")
            .send()
            .await
        {
            if let Ok(versions) = resp.json::<Vec<serde_json::Value>().await {
                if let Some(best) = versions.first() {
                    let name = best["name"].as_str().unwrap_or("unknown").to_string();
                    let ver_id = best["id"].as_str().unwrap_or_default().to_string();
                    let files = best["files"].as_array().cloned().unwrap_or_default();
                    if let Some(primary) = files.iter().find(|f| f["primary"].as_bool().unwrap_or(true)) {
                        resolved.push(ResolvedMod {
                            project_id: dep_project_id.to_string(),
                            source: "modrinth".to_string(),
                            name,
                            version_id: ver_id,
                            download_url: primary["url"].as_str().unwrap_or_default().to_string(),
                            filename: primary["filename"].as_str().unwrap_or_default().to_string(),
                            sha512: primary["hashes"]["sha512"].as_str().map(|s| s.to_string()),
                            is_dependency: true,
                        });
                    }
                }
            }
        }
    }

    Ok(resolved)
}
