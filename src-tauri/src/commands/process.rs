use crate::error::AppError;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_running_instances(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    Ok(state.process_manager.running_instances())
}

#[tauri::command]
pub fn is_instance_running(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<bool, AppError> {
    Ok(state.process_manager.is_running(&instance_id))
}

#[tauri::command]
pub fn kill_game(state: State<'_, AppState>, instance_id: String) -> Result<(), AppError> {
    state.process_manager.kill(&instance_id).map_err(AppError::Internal)?;
    Ok(())
}

#[tauri::command]
pub fn get_game_logs(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<String>, AppError> {
    Ok(state.process_manager.get_logs(&instance_id))
}
