use crate::api::auth;
use crate::db;
use crate::error::AppError;
use crate::AppState;
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Serialize, Clone)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct AccountInfo {
    pub uuid: String,
    pub username: String,
    pub skin_url: Option<String>,
}

/// Start Microsoft login via device code flow.
/// Returns the device code info for the frontend to display.
/// The frontend will call poll_login repeatedly until the user completes auth.
#[tauri::command]
pub async fn start_login(
    _app: AppHandle,
    state: State<'_, AppState>,
) -> Result<DeviceCodeInfo, AppError> {
    let device_code = auth::start_device_code_flow()
        .await
        .map_err(|e| AppError::Auth(format!("Device code error: {e}")))?;

    // Store device code temporarily
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    db::settings::set_setting(&db, "_device_code", &device_code.device_code)?;
    db::settings::set_setting(
        &db,
        "_device_code_expires",
        &device_code.expires_in.to_string(),
    )?;

    Ok(DeviceCodeInfo {
        user_code: device_code.user_code,
        verification_uri: device_code.verification_uri,
        message: device_code.message,
    })
}

/// Poll for login completion. Returns Ok(AccountInfo) when the user completes auth,
/// or Err with status info if still waiting.
#[tauri::command]
pub async fn poll_login(state: State<'_, AppState>) -> Result<AccountInfo, AppError> {
    let device_code = {
        let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::settings::get_setting(&db, "_device_code")?
            .ok_or_else(|| AppError::NotFound("No login in progress".into()))?
    };

    // Poll Microsoft for tokens
    let (msa_token, msa_refresh) = auth::poll_for_token(&device_code)
        .await
        .map_err(|e| AppError::Auth(e.to_string()))?;

    // Xbox auth chain
    let (mc_token, _xuid) = auth::xbox_auth_chain(&msa_token)
        .await
        .map_err(|e| AppError::Auth(e.to_string()))?;

    // Get Minecraft profile
    let profile = auth::get_minecraft_profile(&mc_token)
        .await
        .map_err(|e| AppError::Auth(e.to_string()))?;

    let skin_url = profile.skins.first().map(|s| s.url.clone());

    // Save to database
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let account = db::accounts::Account {
        uuid: profile.id.clone(),
        username: profile.name.clone(),
        access_token: mc_token,
        refresh_token: msa_refresh,
        skin_url,
    };
    db::accounts::upsert_account(&db, &account)?;

    // Clean up
    let _ = db::settings::delete_setting(&db, "_device_code");
    let _ = db::settings::delete_setting(&db, "_device_code_expires");

    Ok(AccountInfo {
        uuid: account.uuid,
        username: account.username,
        skin_url: account.skin_url,
    })
}

#[tauri::command]
pub fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountInfo>, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let accounts = db::accounts::get_all_accounts(&db)?;

    Ok(accounts
        .into_iter()
        .map(|a| AccountInfo {
            uuid: a.uuid,
            username: a.username,
            skin_url: a.skin_url,
        })
        .collect())
}

#[tauri::command]
pub fn remove_account(state: State<'_, AppState>, uuid: String) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    db::accounts::delete_account(&db, &uuid)?;
    Ok(())
}

/// Refresh an account's MC token using the stored MSA refresh token.
/// Returns Ok(AccountInfo) with updated tokens, or Err if refresh fails.
#[tauri::command]
pub async fn refresh_account_token(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<AccountInfo, AppError> {
    let refresh_token = {
        let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let accounts = db::accounts::get_all_accounts(&db)?;
        let account = accounts
            .iter()
            .find(|a| a.uuid == uuid)
            .ok_or_else(|| AppError::NotFound("Account not found".into()))?;
        if account.refresh_token.is_empty() {
            return Err(AppError::Auth("No refresh token stored. Please sign in again.".into()));
        }
        account.refresh_token.clone()
    };

    // Refresh the MSA token
    let (new_msa_token, new_msa_refresh) = auth::refresh_msa_token(&refresh_token)
        .await
        .map_err(|e| AppError::Auth(format!("MSA refresh failed: {e}")))?;

    // Xbox auth chain with new MSA token
    let (mc_token, _xuid) = auth::xbox_auth_chain(&new_msa_token)
        .await
        .map_err(|e| AppError::Auth(format!("Xbox auth failed after refresh: {e}")))?;

    // Get updated profile
    let profile = auth::get_minecraft_profile(&mc_token)
        .await
        .map_err(|e| AppError::Auth(format!("Profile fetch failed after refresh: {e}")))?;

    let skin_url = profile.skins.first().map(|s| s.url.clone());

    // Update tokens in DB
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    db::accounts::update_tokens(&db, &uuid, &mc_token, &new_msa_refresh)?;

    Ok(AccountInfo {
        uuid: profile.id,
        username: profile.name,
        skin_url,
    })
}

#[tauri::command]
pub fn switch_active_account(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<AccountInfo, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    db.execute(
        "UPDATE accounts SET last_used = ?1 WHERE uuid = ?2",
        rusqlite::params![chrono::Utc::now().to_rfc3339(), uuid],
    )?;

    let account = db::accounts::get_all_accounts(&db)?
        .into_iter()
        .find(|a| a.uuid == uuid)
        .ok_or_else(|| AppError::NotFound("Account not found".into()))?;

    Ok(AccountInfo {
        uuid: account.uuid,
        username: account.username,
        skin_url: account.skin_url,
    })
}
