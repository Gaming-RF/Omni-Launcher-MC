use crate::api::auth;
use crate::db;
use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct AccountInfo {
    pub uuid: String,
    pub username: String,
    pub skin_url: Option<String>,
}

#[tauri::command]
pub async fn start_login(state: State<'_, AppState>) -> Result<DeviceCodeInfo, String> {
    let device_code = auth::start_device_code_flow()
        .await
        .map_err(|e| e.to_string())?;

    // Store device_code temporarily in settings for polling
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::settings::set_setting(&db, "_device_code", &device_code.device_code)
        .map_err(|e| e.to_string())?;

    Ok(DeviceCodeInfo {
        user_code: device_code.user_code,
        verification_uri: device_code.verification_uri,
        message: device_code.message,
    })
}

#[tauri::command]
pub async fn poll_login(state: State<'_, AppState>) -> Result<AccountInfo, String> {
    // Get stored device code
    let device_code = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db::settings::get_setting(&db, "_device_code")
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "No login in progress".to_string())?
    };

    // Poll Microsoft for tokens
    let (msa_token, _refresh) = auth::poll_for_token(&device_code)
        .await
        .map_err(|e| e.to_string())?;

    // Xbox auth chain
    let (mc_token, _xuid) = auth::xbox_auth_chain(&msa_token)
        .await
        .map_err(|e| e.to_string())?;

    // Get Minecraft profile
    let profile = auth::get_minecraft_profile(&mc_token)
        .await
        .map_err(|e| e.to_string())?;

    let skin_url = profile.skins.first().map(|s| s.url.clone());

    // Save to database
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let account = db::accounts::Account {
        uuid: profile.id.clone(),
        username: profile.name.clone(),
        access_token: mc_token,
        refresh_token: String::new(), // TODO: Store MSA refresh token
        skin_url,
    };
    db::accounts::upsert_account(&db, &account).map_err(|e| e.to_string())?;

    // Clean up device code
    let _ = db::settings::delete_setting(&db, "_device_code");

    Ok(AccountInfo {
        uuid: account.uuid,
        username: account.username,
        skin_url: account.skin_url,
    })
}

#[tauri::command]
pub fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountInfo>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let accounts = db::accounts::get_all_accounts(&db).map_err(|e| e.to_string())?;

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
pub fn remove_account(state: State<'_, AppState>, uuid: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::accounts::delete_account(&db, &uuid).map_err(|e| e.to_string())?;
    Ok(())
}
