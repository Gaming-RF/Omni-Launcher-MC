use crate::AppState;
use crate::commands::instances::InstanceListItem;
use crate::db;
use serde::{Deserialize, Serialize};
use tauri::State;

// ── Types ────────────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub mods: Vec<TemplateMod>,
    pub is_custom: bool,
    pub category: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplateMod {
    pub name: String,
    pub slug: String,
    pub source: String,
    pub project_id: String,
    pub description: String,
}

// ── Built-in templates ───────────────────────────────────────────

fn builtin_templates() -> Vec<TemplateInfo> {
    vec![
        TemplateInfo {
            id: "vanilla-latest".into(),
            name: "Vanilla Latest".into(),
            description: "Pure Minecraft, no mods".into(),
            icon: "🎮".into(),
            game_version: "latest".into(),
            loader: "vanilla".into(),
            loader_version: None,
            mods: vec![],
            is_custom: false,
            category: "vanilla".into(),
        },
        TemplateInfo {
            id: "fabric-performance".into(),
            name: "Fabric Performance".into(),
            description: "Fabric + Sodium + Lithium + Starlight for maximum FPS".into(),
            icon: "⚡".into(),
            game_version: "latest".into(),
            loader: "fabric".into(),
            loader_version: None,
            mods: vec![
                TemplateMod {
                    name: "Sodium".into(),
                    slug: "sodium".into(),
                    source: "modrinth".into(),
                    project_id: "AANobbMI".into(),
                    description: "Modern rendering engine".into(),
                },
                TemplateMod {
                    name: "Lithium".into(),
                    slug: "lithium".into(),
                    source: "modrinth".into(),
                    project_id: "gvQqBUqZ".into(),
                    description: "Game logic/server optimizations".into(),
                },
                TemplateMod {
                    name: "Starlight".into(),
                    slug: "starlight".into(),
                    source: "modrinth".into(),
                    project_id: "H8CaAYZC".into(),
                    description: "Lighting engine rewrite".into(),
                },
            ],
            is_custom: false,
            category: "performance".into(),
        },
        TemplateInfo {
            id: "forge-create".into(),
            name: "Create Mod".into(),
            description: "Forge + Create + Flywheel for mechanical builds".into(),
            icon: "⚙️".into(),
            game_version: "latest".into(),
            loader: "forge".into(),
            loader_version: None,
            mods: vec![TemplateMod {
                name: "Create".into(),
                slug: "create".into(),
                source: "modrinth".into(),
                project_id: "LvNZAw2M".into(),
                description: "Technology mod with rotating machinery".into(),
            }],
            is_custom: false,
            category: "modded".into(),
        },
        TemplateInfo {
            id: "fabric-optifine-alt".into(),
            name: "Fabric Visual".into(),
            description: "Fabric + Iris + Sodium for shaders + performance".into(),
            icon: "✨".into(),
            game_version: "latest".into(),
            loader: "fabric".into(),
            loader_version: None,
            mods: vec![
                TemplateMod {
                    name: "Iris Shaders".into(),
                    slug: "iris".into(),
                    source: "modrinth".into(),
                    project_id: "YL57xq9U".into(),
                    description: "Shader mod compatible with OptiFine shaders".into(),
                },
                TemplateMod {
                    name: "Sodium".into(),
                    slug: "sodium".into(),
                    source: "modrinth".into(),
                    project_id: "AANobbMI".into(),
                    description: "Rendering engine".into(),
                },
            ],
            is_custom: false,
            category: "modded".into(),
        },
        TemplateInfo {
            id: "quilt-latest".into(),
            name: "Quilt Latest".into(),
            description: "Quilt modloader with QSL".into(),
            icon: "🧵".into(),
            game_version: "latest".into(),
            loader: "quilt".into(),
            loader_version: None,
            mods: vec![],
            is_custom: false,
            category: "vanilla".into(),
        },
        TemplateInfo {
            id: "modpack-adventure".into(),
            name: "Adventure Pack".into(),
            description: "Fabric + adventure/exploration mods".into(),
            icon: "🗺️".into(),
            game_version: "latest".into(),
            loader: "fabric".into(),
            loader_version: None,
            mods: vec![
                TemplateMod {
                    name: "Terralith".into(),
                    slug: "terralith".into(),
                    source: "modrinth".into(),
                    project_id: "8iL5Jf6k".into(),
                    description: "Overworld terrain overhaul".into(),
                },
                TemplateMod {
                    name: "Better Animals Plus".into(),
                    slug: "betteranimalsplus".into(),
                    source: "modrinth".into(),
                    project_id: "HHHxQjgS".into(),
                    description: "New animals and creatures".into(),
                },
            ],
            is_custom: false,
            category: "modded".into(),
        },
    ]
}

// ── DB helpers for custom_templates ──────────────────────────────

fn get_custom_templates_db(db: &rusqlite::Connection) -> Result<Vec<TemplateInfo>, String> {
    let mut stmt = db
        .prepare(
            "SELECT id, name, description, game_version, loader, loader_version, mods_json, icon \
             FROM custom_templates ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let mods_json: Option<String> = row.get(6)?;
            let mods: Vec<TemplateMod> = mods_json
                .as_deref()
                .and_then(|j| serde_json::from_str(j).ok())
                .unwrap_or_default();

            Ok(TemplateInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                game_version: row.get(3)?,
                loader: row.get(4)?,
                loader_version: row.get(5)?,
                mods,
                is_custom: true,
                category: "custom".into(),
                icon: row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "📦".into()),
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// ── Commands ─────────────────────────────────────────────────────

/// Returns all templates (builtin + custom).
#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> Result<Vec<TemplateInfo>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut templates = builtin_templates();
    let custom = get_custom_templates_db(&db)?;
    templates.extend(custom);
    Ok(templates)
}

/// Returns only custom templates from the database.
#[tauri::command]
pub fn list_custom_templates(state: State<'_, AppState>) -> Result<Vec<TemplateInfo>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    get_custom_templates_db(&db)
}

/// Creates a new instance from a template. Resolves "latest" to a concrete version.
#[tauri::command]
pub fn create_instance_from_template(
    state: State<'_, AppState>,
    template_id: String,
    name: String,
    game_version: Option<String>,
) -> Result<InstanceListItem, String> {
    // Find the template (builtin first, then custom DB)
    let template = builtin_templates()
        .into_iter()
        .find(|t| t.id == template_id)
        .or_else(|| {
            let db = state.db.lock().ok()?;
            get_custom_templates_db(&db).ok()?.into_iter().find(|t| t.id == template_id)
        })
        .ok_or_else(|| format!("Template '{}' not found", template_id))?;

    // Resolve game version: prefer explicit override, then check "latest"
    let resolved_version = game_version.unwrap_or_else(|| {
        if template.game_version == "latest" {
            // Use a sensible default; the actual latest will be resolved at launch time
            "1.21.4".into()
        } else {
            template.game_version.clone()
        }
    });

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let instance = db::instances::create_instance(
        &db,
        db::instances::CreateInstanceParams {
            name,
            game_version: resolved_version,
            loader: template.loader,
            loader_version: template.loader_version,
            icon: Some(template.icon),
            java_args: None,
            // Default 4 GB
            allocated_memory_mb: 4096,
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(InstanceListItem {
        id: instance.id,
        name: instance.name,
        game_version: instance.game_version,
        loader: instance.loader,
        loader_version: instance.loader_version,
        icon: instance.icon,
        created_at: instance.created_at,
        last_played: instance.last_played,
        play_time_secs: instance.play_time_secs,
        allocated_memory_mb: instance.allocated_memory_mb,
    })
}

/// Saves an existing instance as a custom template.
#[tauri::command]
pub fn save_as_template(
    state: State<'_, AppState>,
    instance_id: String,
    template_name: String,
    description: String,
) -> Result<TemplateInfo, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Fetch the source instance
    let instance = db::instances::get_instance(&db, &instance_id)
        .map_err(|e| e.to_string())?
        .ok_or("Instance not found")?;

    // Fetch installed mods for the instance and turn them into TemplateMod stubs
    let mods = db::mods::get_instance_mods(&db, &instance_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|m| TemplateMod {
            name: m.name.clone(),
            slug: m.mod_id.clone(),
            source: m.source.clone(),
            project_id: m.mod_id.clone(),
            description: String::new(),
        })
        .collect::<Vec<_>>();

    let mods_json = serde_json::to_string(&mods).unwrap_or_else(|_| "[]".into());
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let icon = instance.icon.unwrap_or_else(|| "📦".into());

    db.execute(
        "INSERT INTO custom_templates (id, name, description, game_version, loader, loader_version, mods_json, icon, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            id,
            template_name,
            description,
            instance.game_version,
            instance.loader,
            instance.loader_version,
            mods_json,
            icon,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(TemplateInfo {
        id,
        name: template_name,
        description,
        game_version: instance.game_version,
        loader: instance.loader,
        loader_version: instance.loader_version,
        mods,
        is_custom: true,
        category: "custom".into(),
        icon,
    })
}

/// Deletes a custom template by ID.
#[tauri::command]
pub fn delete_custom_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let rows = db
        .execute(
            "DELETE FROM custom_templates WHERE id = ?1",
            rusqlite::params![template_id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err("Template not found".into());
    }
    Ok(())
}
