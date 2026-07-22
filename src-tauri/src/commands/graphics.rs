use crate::utils::paths::data_dir;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphicsSettings {
    pub render_distance: u32,
    pub simulation_distance: u32,
    pub fov: f32,
    pub gui_scale: u32,
    pub max_fps: u32,
    pub vsync: bool,
    pub graphics: String,
    pub smooth_lighting: u32,
    pub particles: u32,
    pub entity_shadows: bool,
    pub biome_blend: u32,
    pub clouds: String,
    pub fullscreen: bool,
    pub mipmap_levels: u32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            render_distance: 12,
            simulation_distance: 8,
            fov: 70.0,
            gui_scale: 3,
            max_fps: 0,
            vsync: false,
            graphics: "fancy".into(),
            smooth_lighting: 2,
            particles: 0,
            entity_shadows: true,
            biome_blend: 2,
            clouds: "fancy".into(),
            fullscreen: false,
            mipmap_levels: 4,
        }
    }
}

fn settings_path(instance_id: &str) -> std::path::PathBuf {
    data_dir()
        .join("instances")
        .join(instance_id)
        .join("graphics.json")
}

#[tauri::command]
pub fn get_graphics_settings(instance_id: String) -> Result<GraphicsSettings, String> {
    let path = settings_path(&instance_id);
    if !path.exists() {
        return Ok(GraphicsSettings::default());
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_graphics_settings(
    instance_id: String,
    settings: GraphicsSettings,
) -> Result<(), String> {
    let path = settings_path(&instance_id);
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_graphics_settings(instance_id: String) -> Result<(), String> {
    let settings = get_graphics_settings(instance_id.clone())?;
    let options_path = data_dir()
        .join("instances")
        .join(&instance_id)
        .join(".minecraft")
        .join("options.txt");

    // Read existing options.txt or create empty
    let mut options: std::collections::HashMap<String, String> = if options_path.exists() {
        let data = fs::read_to_string(&options_path).map_err(|e| e.to_string())?;
        data.lines()
            .filter_map(|line| {
                let (k, v) = line.split_once(':')?;
                Some((k.to_string(), v.to_string()))
            })
            .collect()
    } else {
        std::collections::HashMap::new()
    };

    options.insert("renderDistance".into(), settings.render_distance.to_string());
    options.insert("simulationDistance".into(), settings.simulation_distance.to_string());
    options.insert("fov".into(), settings.fov.to_string());
    options.insert("guiScale".into(), settings.gui_scale.to_string());
    options.insert("maxFps".into(), settings.max_fps.to_string());
    options.insert("enableVsync".into(), settings.vsync.to_string());
    options.insert("graphicsMode".into(), match settings.graphics.as_str() {
        "fast" => "0",
        _ => "1",
    }.to_string());
    options.insert("ao".into(), settings.smooth_lighting.to_string());
    options.insert("particleOption".into(), settings.particles.to_string());
    options.insert("entityShadows".into(), settings.entity_shadows.to_string());
    options.insert("biomeBlendRadius".into(), settings.biome_blend.to_string());
    options.insert("renderClouds".into(), match settings.clouds.as_str() {
        "off" => "false",
        "fast" => "fast",
        _ => "true",
    }.to_string());
    options.insert("fullscreen".into(), settings.fullscreen.to_string());
    options.insert("mipmapLevels".into(), settings.mipmap_levels.to_string());

    let content: String = options
        .iter()
        .map(|(k, v)| format!("{}:{}", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    fs::create_dir_all(options_path.parent().unwrap()).ok();
    fs::write(&options_path, content).map_err(|e| e.to_string())
}
