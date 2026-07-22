use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Clone, Debug)]
pub struct RunningInstanceInfo {
    pub instance_id: String,
    pub instance_name: String,
    pub is_running: bool,
}

#[tauri::command]
pub fn get_all_running_instances(
    state: State<'_, AppState>,
) -> Result<Vec<RunningInstanceInfo>, String> {
    let ids = state.process_manager.running_instances();
    let mut result = Vec::new();
    for id in ids {
        let name = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            crate::db::instances::get_instance(&db, &id)
                .ok()
                .flatten()
                .map(|i| i.name)
                .unwrap_or_else(|| id.clone())
        };
        result.push(RunningInstanceInfo {
            instance_id: id,
            instance_name: name,
            is_running: true,
        });
    }
    Ok(result)
}

#[tauri::command]
pub fn terminate_instance(state: State<'_, AppState>, instance_id: String) -> Result<(), String> {
    state.process_manager.kill(&instance_id)
}

#[tauri::command]
pub fn terminate_all_instances(state: State<'_, AppState>) -> Result<u32, String> {
    let ids = state.process_manager.running_instances();
    let count = ids.len() as u32;
    for id in ids {
        state.process_manager.kill(&id).ok();
    }
    Ok(count)
}
