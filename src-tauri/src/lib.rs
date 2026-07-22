#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::Manager;

pub mod api;
pub mod commands;
pub mod db;
pub mod platforms;
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
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            commands::auth::switch_active_account,
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
            // Modpack browsing + one-click install
            commands::minecraft::search_modpacks_modrinth,
            commands::minecraft::search_modpacks_curseforge,
            commands::minecraft::get_modpack_versions_modrinth,
            commands::minecraft::download_and_install_modpack,
            commands::platform::search_mods_unified,
            commands::platform::get_mod_versions_unified,
            commands::platform::get_mod_details_unified,
            // Mod update checker
            commands::loaders::check_mod_updates,
            // Resource packs & shaders
            commands::minecraft::list_installed_packs,
            commands::minecraft::toggle_pack,
            commands::minecraft::delete_pack,
            // Import from other launchers
            commands::import::scan_launcher_instances,
            commands::import::import_launcher_instance,
            // Desktop shortcuts
            commands::shortcuts::create_desktop_shortcut,
            commands::shortcuts::get_shortcut_default_dir,
            // Worlds & servers
            commands::worlds::get_instance_worlds,
            commands::worlds::add_server,
            commands::worlds::edit_server,
            commands::worlds::remove_server,
            commands::worlds::ping_server,
            commands::worlds::delete_world,
            commands::worlds::rename_world,
            commands::worlds::backup_world,
            // Skins
            commands::skins::get_skin_info,
            commands::skins::upload_skin,
            commands::skins::reset_skin,
            commands::skins::get_capes,
            // Instance hooks
            commands::hooks::get_instance_hooks,
            commands::hooks::update_instance_hooks,
            // Advanced logs
            commands::logs::get_log_files,
            commands::logs::read_log_cursor,
            commands::logs::read_log_file,
            commands::logs::delete_log_file,
            commands::logs::delete_all_logs,
            commands::logs::get_log_size,
            // Instance groups
            commands::groups::list_groups,
            commands::groups::create_group,
            commands::groups::delete_group,
            commands::groups::rename_group,
            commands::groups::update_group_color,
            commands::groups::assign_instance_to_group,
            commands::groups::remove_instance_from_group,
            commands::groups::get_group_instances,
            // Mirror / CDN switching
            commands::mirrors::list_mirrors,
            commands::mirrors::get_mirror,
            commands::mirrors::set_mirror,
            commands::mirrors::test_mirror,
            commands::mirrors::test_all_mirrors,
            commands::mirrors::resolve_download_url,
            // Instance templates
            commands::templates::list_templates,
            commands::templates::list_custom_templates,
            commands::templates::create_instance_from_template,
            commands::templates::save_as_template,
            commands::templates::delete_custom_template,
            // Modpack export (.mrpack)
            commands::mrpack_export::export_mrpack,
            commands::mrpack_export::export_mrpack_to_path,
            // Resource packs & shaders management
            commands::resource_packs::list_resource_packs,
            commands::resource_packs::list_shaders,
            commands::resource_packs::toggle_resource_pack,
            commands::resource_packs::toggle_shader,
            commands::resource_packs::delete_resource_pack,
            commands::resource_packs::delete_shader,
            commands::resource_packs::open_resource_packs_folder,
            commands::resource_packs::open_shaders_folder,
            // Screenshots
            commands::screenshots::list_screenshots,
            commands::screenshots::delete_screenshot,
            commands::screenshots::open_screenshots_folder,
            commands::screenshots::export_screenshot,
            // Modloader matrix
            commands::modloader_matrix::get_modloader_matrix,
            commands::modloader_matrix::get_instance_modloader_matrix,
            // Graphics settings
            commands::graphics::get_graphics_settings,
            commands::graphics::update_graphics_settings,
            commands::graphics::apply_graphics_settings,
            // Resource library
            commands::library::list_library_items,
            commands::library::import_to_library,
            commands::library::link_library_to_instance,
            commands::library::unlink_library_from_instance,
            commands::library::cleanup_library,
            // Resource categorization
            commands::categorize::categorize_instance_mods,
            // Multi-instance launch
            commands::multi_launch::get_all_running_instances,
            commands::multi_launch::terminate_instance,
            commands::multi_launch::terminate_all_instances,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OmniLauncherMC");
}
