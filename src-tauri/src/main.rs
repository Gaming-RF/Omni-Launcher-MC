// OmniLauncherMC - Tauri application entry point
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod commands;
mod db;
mod models;
mod utils;

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
            // Initialize database on app start
            let app_data = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data).ok();
            let db_path = app_data.join("omni.db");
            db::init(&db_path).expect("Failed to initialize database");
            log::info!("Database initialized at {:?}", db_path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::login_start,
            commands::auth::login_poll,
            commands::auth::get_profile,
            commands::auth::logout,
            commands::auth::list_accounts,
            commands::instances::list_instances,
            commands::instances::create_instance,
            commands::instances::delete_instance,
            commands::instances::update_instance,
            commands::minecraft::get_version_manifest,
            commands::minecraft::get_version_details,
            commands::minecraft::download_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OmniLauncherMC");
}
