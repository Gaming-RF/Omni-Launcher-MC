use serde::{Deserialize, Serialize};
use tauri::command;

/// Represents a game instance in the launcher
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub mod_loader: Option<String>,
    pub mod_loader_version: Option<String>,
    pub icon: Option<String>,
    pub created_at: i64,
    pub last_played: Option<i64>,
    pub play_time_seconds: u64,
    pub source: Option<String>,
    pub source_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInstanceParams {
    pub name: String,
    pub game_version: String,
    pub mod_loader: Option<String>,
    pub mod_loader_version: Option<String>,
    pub icon: Option<String>,
}

/// Lists all instances ordered by last played.
#[command]
pub async fn list_instances() -> Result<Vec<Instance>, String> {
    // TODO: Query from SQLite, return all instances
    Err("Not yet implemented".to_string())
}

/// Creates a new instance and its directory structure.
#[command]
pub async fn create_instance(params: CreateInstanceParams) -> Result<Instance, String> {
    // TODO: Generate UUID, create directory, insert into DB
    // Instance dir: <app_data>/instances/<uuid>/
    // Subdirs: .minecraft/, mods/, config/, saves/
    Err("Not yet implemented".to_string())
}

/// Deletes an instance and its files.
#[command]
pub async fn delete_instance(id: String) -> Result<(), String> {
    // TODO: Remove from DB, delete instance directory
    Err("Not yet implemented".to_string())
}

/// Updates instance metadata (name, icon, mod loader, etc.)
#[command]
pub async fn update_instance(id: String, name: Option<String>, icon: Option<String>) -> Result<Instance, String> {
    // TODO: Update DB record
    Err("Not yet implemented".to_string())
}
