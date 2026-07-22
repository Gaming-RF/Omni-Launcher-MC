use crate::utils::paths::data_dir;
use serde::Serialize;
use std::fs;

#[derive(Serialize, Clone, Debug)]
pub struct ModCategory {
    pub category: String,
    pub subcategories: Vec<String>,
    pub description: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct CategorizedMod {
    pub mod_id: String,
    pub name: String,
    pub file_name: String,
    pub detected_category: ModCategory,
    pub detected_loaders: Vec<String>,
    pub detected_game_versions: Vec<String>,
    pub compatibility: String,
}

const KNOWN_CATEGORIES: &[(&str, &str, &[&str])] = &[
    (
        "performance",
        "Performance",
        &[
            "sodium",
            "lithium",
            "starlight",
            "ferritecore",
            "krypton",
            "lazydfu",
            "c2me",
        ],
    ),
    (
        "optimization",
        "Graphics Optimization",
        &["iris", "canvas", "optifine", "embeddium", "oculus"],
    ),
    (
        "worldgen",
        "World Generation",
        &[
            "terralith",
            "biomesoplenty",
            "oh-the-biomes",
            "tectonic",
            "regions-unexplored",
        ],
    ),
    (
        "tech",
        "Technology",
        &[
            "create",
            "mekanism",
            "appliedenergistics",
            "refined-storage",
            "immersive-engineering",
            "ae2",
        ],
    ),
    (
        "magic",
        "Magic",
        &[
            "botania",
            "ars-nouveau",
            "blood-magic",
            "occultism",
            "evilcraft",
        ],
    ),
    (
        "adventure",
        "Adventure",
        &[
            "twilight-forest",
            "aether",
            "dungeons",
            "better-dungeons",
            "yungs",
        ],
    ),
    (
        "utility",
        "Utility",
        &[
            "jei",
            "rei",
            "wthit",
            "jade",
            "emi",
            "crafttweaker",
            "kubejs",
        ],
    ),
    (
        "library",
        "Library/API",
        &[
            "fabric-api",
            "forge",
            "cloth-config",
            "architectury",
            "fabric-language-kotlin",
            "quilted-fabric-api",
        ],
    ),
    (
        "qol",
        "Quality of Life",
        &[
            "appleskin",
            "journeymap",
            "minimap",
            "inventory-hud",
            "xaeros",
            "mouse-tweaks",
        ],
    ),
    (
        "cosmetic",
        "Cosmetic",
        &[
            "fresh-animations",
            "entity-texture-features",
            "continuity",
            "lambdynamiclights",
        ],
    ),
];

fn categorize_mod_id(mod_id: &str) -> ModCategory {
    let lower = mod_id.to_lowercase();
    for &(cat_id, cat_name, keywords) in KNOWN_CATEGORIES {
        for kw in keywords {
            if lower.contains(kw) {
                return ModCategory {
                    category: cat_id.to_string(),
                    subcategories: vec![],
                    description: cat_name.to_string(),
                };
            }
        }
    }
    ModCategory {
        category: "other".into(),
        subcategories: vec![],
        description: "Other".into(),
    }
}

fn read_jar_metadata(path: &std::path::Path) -> Option<(String, Vec<String>, Vec<String>)> {
    let file = fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    // Try fabric.mod.json
    if let Ok(mut f) = archive.by_name("fabric.mod.json") {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut f, &mut s).ok()?;
        let json: serde_json::Value = serde_json::from_str(&s).ok()?;
        let mod_id = json.get("id")?.as_str()?.to_string();
        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&mod_id)
            .to_string();
        let gv: Vec<String> = json
            .get("depends")
            .and_then(|d| d.get("minecraft"))
            .into_iter()
            .flat_map(|_| vec![]) // fabric doesn't list game versions in mod.json
            .collect();
        return Some((name, vec!["fabric".into()], gv));
    }

    // Try META-INF/mods.toml (Forge/NeoForge)
    if let Ok(mut f) = archive.by_name("META-INF/mods.toml") {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut f, &mut s).ok()?;
        let mod_id = s
            .lines()
            .find(|l| l.trim_start().starts_with("modId"))
            .and_then(|l| l.split_once('='))
            .map(|(_, v)| v.trim().trim_matches('"').to_string())
            .unwrap_or_default();
        let name = s
            .lines()
            .find(|l| l.trim_start().starts_with("displayName"))
            .and_then(|l| l.split_once('='))
            .map(|(_, v)| v.trim().trim_matches('"').to_string())
            .unwrap_or_else(|| mod_id.clone());
        if !mod_id.is_empty() {
            return Some((name, vec!["forge".into()], vec![]));
        }
    }

    // Try quilt.mod.json
    if let Ok(mut f) = archive.by_name("quilt.mod.json") {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut f, &mut s).ok()?;
        let json: serde_json::Value = serde_json::from_str(&s).ok()?;
        let mod_id = json
            .get("quilt_loader")
            .and_then(|ql| ql.get("id"))
            .and_then(|v| v.as_str())?
            .to_string();
        let name = json
            .get("quilt_loader")
            .and_then(|ql| ql.get("metadata"))
            .and_then(|m| m.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or(&mod_id)
            .to_string();
        return Some((name, vec!["quilt".into()], vec![]));
    }

    None
}

#[tauri::command]
pub fn categorize_instance_mods(instance_id: String) -> Result<Vec<CategorizedMod>, String> {
    let mods_dir = data_dir()
        .join("instances")
        .join(&instance_id)
        .join(".minecraft")
        .join("mods");
    if !mods_dir.exists() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    for entry in fs::read_dir(&mods_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".jar") {
            continue;
        }
        let path = entry.path();
        let (display_name, loaders, game_versions) = read_jar_metadata(&path)
            .unwrap_or_else(|| (name.trim_end_matches(".jar").to_string(), vec![], vec![]));
        let category = categorize_mod_id(&display_name.to_lowercase());
        result.push(CategorizedMod {
            mod_id: display_name.to_lowercase().replace(' ', "-"),
            name: display_name,
            file_name: name,
            detected_category: category,
            detected_loaders: loaders,
            detected_game_versions: game_versions,
            compatibility: "unknown".into(),
        });
    }
    result.sort_by(|a, b| {
        a.detected_category
            .category
            .cmp(&b.detected_category.category)
    });
    Ok(result)
}
