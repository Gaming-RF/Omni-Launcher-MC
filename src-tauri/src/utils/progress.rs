// Download progress tracking via Tauri events.
// Emits structured progress events that the frontend can subscribe to.
//
// Event name: "download-progress"
// Payload: ProgressEvent { task_id, phase, current, total, message }
//
// Phases:
//   "version_json"  - Downloading version JSON
//   "client_jar"    - Downloading client JAR
//   "libraries"     - Downloading libraries (current/total = lib index)
//   "assets"        - Downloading assets (current/total = asset index)
//   "loader"        - Installing mod loader
//   "mod"           - Downloading a mod file
//   "modpack"       - Downloading modpack files
//   "java"          - Downloading Java runtime
//   "complete"      - Task finished
//   "error"         - Task failed

use serde::Serialize;
use tauri::Emitter;

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub task_id: String,
    pub phase: String,
    pub current: u64,
    pub total: u64,
    pub message: String,
}

/// Emit a progress event to the frontend.
pub fn emit(app: &tauri::AppHandle, event: ProgressEvent) {
    if let Err(e) = app.emit("download-progress", &event) {
        log::warn!("Failed to emit progress event: {}", e);
    }
}

/// Convenience: emit a phase start event.
pub fn phase_start(app: &tauri::AppHandle, task_id: &str, phase: &str, message: &str) {
    emit(
        app,
        ProgressEvent {
            task_id: task_id.to_string(),
            phase: phase.to_string(),
            current: 0,
            total: 0,
            message: message.to_string(),
        },
    );
}

/// Convenience: emit a progress update.
pub fn update(app: &tauri::AppHandle, task_id: &str, phase: &str, current: u64, total: u64) {
    emit(
        app,
        ProgressEvent {
            task_id: task_id.to_string(),
            phase: phase.to_string(),
            current,
            total,
            message: format!("{}/{}", current, total),
        },
    );
}

/// Convenience: emit completion.
pub fn complete(app: &tauri::AppHandle, task_id: &str, message: &str) {
    emit(
        app,
        ProgressEvent {
            task_id: task_id.to_string(),
            phase: "complete".to_string(),
            current: 1,
            total: 1,
            message: message.to_string(),
        },
    );
}

/// Convenience: emit error.
pub fn error(app: &tauri::AppHandle, task_id: &str, message: &str) {
    emit(
        app,
        ProgressEvent {
            task_id: task_id.to_string(),
            phase: "error".to_string(),
            current: 0,
            total: 0,
            message: message.to_string(),
        },
    );
}
