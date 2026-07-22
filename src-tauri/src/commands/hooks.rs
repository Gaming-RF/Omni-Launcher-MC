use crate::error::AppError;
use crate::db;
use crate::AppState;
use serde::Serialize;
use tauri::State;

/// Instance hook configuration.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct InstanceHooks {
    pub pre_launch_cmd: Option<String>,
    pub post_exit_cmd: Option<String>,
    pub hook_env_vars: Option<String>, // JSON string of key-value pairs
}

/// Get hooks for an instance.
#[tauri::command]
pub fn get_instance_hooks(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<InstanceHooks, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let instance = db::instances::get_instance(&db, &instance_id)
        ?
        .ok_or("Instance not found")?;

    Ok(InstanceHooks {
        pre_launch_cmd: instance.pre_launch_cmd,
        post_exit_cmd: instance.post_exit_cmd,
        hook_env_vars: instance.hook_env_vars,
    })
}

/// Update hooks for an instance.
#[tauri::command]
pub fn update_instance_hooks(
    state: State<'_, AppState>,
    instance_id: String,
    hooks: InstanceHooks,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    db.execute(
        "UPDATE instances SET pre_launch_cmd = ?1, post_exit_cmd = ?2, hook_env_vars = ?3 WHERE id = ?4",
        rusqlite::params![
            hooks.pre_launch_cmd,
            hooks.post_exit_cmd,
            hooks.hook_env_vars,
            instance_id
        ],
    )
    ?;

    Ok(())
}

/// Execute a pre-launch hook command.
pub async fn execute_pre_launch(instance_id: &str, hooks: &InstanceHooks) -> Result<(), AppError> {
    if let Some(ref cmd) = hooks.pre_launch_cmd {
        if cmd.is_empty() {
            return Ok(());
        }

        log::info!("Executing pre-launch hook for {}: {}", instance_id, cmd);

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .await
            .map_err(|e| format!("Pre-launch hook failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Internal(format!("{}", "")));
        }
    }
    Ok(())
}

/// Execute a post-exit hook command.
pub async fn execute_post_exit(instance_id: &str, hooks: &InstanceHooks) -> Result<(), AppError> {
    if let Some(ref cmd) = hooks.post_exit_cmd {
        if cmd.is_empty() {
            return Ok(());
        }

        log::info!("Executing post-exit hook for {}: {}", instance_id, cmd);

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .await
            .map_err(|e| format!("Post-exit hook failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("Post-exit hook warning: {}", stderr);
        }
    }
    Ok(())
}
