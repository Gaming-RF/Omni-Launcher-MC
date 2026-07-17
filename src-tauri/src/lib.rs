#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::Manager;

pub mod api;
pub mod commands;
pub mod db;
pub mod utils;

/// Application state shared across commands via Tauri managed state.
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    /// Shared HTTP client with connection pooling.
    pub http: reqwest::Client,
    /// App handle for emitting events (progress, notifications).
    pub app_handle: Mutex<Option<tauri::AppHandle>>,
    /// Tracks running game processes, captures stdout/stderr.
    pub process_manager: utils::process_manager::ProcessManager,
}

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            // Initialize data directories
            let data_dir = utils::paths::data_dir();
            std::fs::create_dir_all(&data_dir)?;
            std::fs::create_dir_all(data_dir.join("instances"))?;
            std::fs::create_dir_all(data_dir.join("versions"))?;
            std::fs::create_dir_all(data_dir.join("libraries"))?;
            std::fs::create_dir_all(data_dir.join("assets"))?;

            // Initialize database
            let db_path = utils::paths::db_path();
            let conn = rusqlite::Connection::open(&db_path)?;
            db::migrations::run_migrations(&conn)?;

            // Build shared HTTP client with connection pooling
            let http_client = reqwest::Client::builder()
                .user_agent("OmniLauncherMC/0.1.0 (github.com/OmniLauncherMC)")
                .timeout(std::time::Duration::from_secs(30))
                .pool_max_idle_per_host(8)
                .build()
                .expect("Failed to build HTTP client");

            // Store connection in managed state
            app.manage(AppState {
                db: Mutex::new(conn),
                http: http_client,
                app_handle: Mutex::new(Some(app.handle().clone())),
                process_manager: utils::process_manager::ProcessManager::new(),
            });

            log::info!("OmniLauncherMC initialized. Data dir: {:?}", data_dir);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            commands::auth::start_login,
            commands::auth::poll_login,
            commands::auth::get_accounts,
            commands::auth::remove_account,
            // Instance commands
            commands::instances::get_instances,
            commands::instances::create_instance,
            commands::instances::delete_instance,
            commands::instances::update_instance,
            // Minecraft commands
            commands::minecraft::get_version_manifest,
            commands::minecraft::launch_game,
            commands::minecraft::launch_game_offline,
            commands::minecraft::prepare_instance,
            commands::minecraft::check_java,
            commands::minecraft::modrinth_search,
            commands::minecraft::curseforge_search,
            // Settings commands
            commands::instances::get_settings,
            commands::instances::update_setting,
            // Loader commands
            commands::loaders::get_fabric_loader_versions,
            commands::loaders::get_quilt_loader_versions,
            commands::loaders::get_forge_versions,
            commands::loaders::get_neoforge_versions,
            commands::loaders::install_fabric_loader,
            commands::loaders::install_quilt_loader,
            commands::loaders::install_forge_loader,
            commands::loaders::install_neoforge_loader,
            // Mod management commands
            commands::loaders::get_instance_mods,
            commands::loaders::install_mod_from_modrinth,
            commands::loaders::install_mod_from_curseforge,
            commands::loaders::install_mod,
            commands::loaders::toggle_mod_enabled,
            commands::loaders::remove_mod,
            commands::loaders::get_modrinth_versions,
            commands::loaders::get_curseforge_versions,
            // Modpack import commands
            commands::loaders::parse_mrpack_file,
            commands::loaders::parse_cf_modpack_file,
            commands::loaders::install_mrpack_modpack,
            // Java management commands
            commands::java::get_required_java_version,
            commands::java::ensure_java_for_mc,
            commands::java::download_java_version,
            // Process management commands
            commands::process::get_running_instances,
            commands::process::is_instance_running,
            commands::process::kill_game,
            commands::process::get_game_logs,
            // Instance management commands
            commands::instances::duplicate_instance,
            // Share commands
            commands::instances::export_instance_share,
            commands::instances::import_instance_share,
            // Aggregated search
            commands::minecraft::aggregated_search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OmniLauncherMC");
}
