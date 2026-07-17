#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod commands;
mod db;
mod utils;

use std::sync::Mutex;
use tauri::Manager;

/// Application state shared across commands via Tauri managed state.
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
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

            // Store connection in managed state
            app.manage(AppState {
                db: Mutex::new(conn),
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
            commands::minecraft::prepare_instance,
            commands::minecraft::check_java,
            // Modrinth commands
            commands::minecraft::modrinth_search,
            // CurseForge commands
            commands::minecraft::curseforge_search,
            // Settings commands
            commands::instances::get_settings,
            commands::instances::update_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OmniLauncherMC");
}
