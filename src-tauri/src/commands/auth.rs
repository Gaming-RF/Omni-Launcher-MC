use crate::api::auth;
use crate::db;
use crate::error::AppError;
use crate::AppState;
use serde::Serialize;
use std::net::TcpListener as StdTcpListener;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, State};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Serialize, Clone)]
pub struct BrowserLoginInfo {
    pub authorization_url: String,
    pub redirect_uri: String,
}

#[derive(Serialize, Clone)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
    pub message: String,
    pub interval: u32,
    pub expires_in: u32,
}

#[derive(Serialize, Clone)]
pub struct AccountInfo {
    pub uuid: String,
    pub username: String,
    pub skin_url: Option<String>,
}

pub struct PendingBrowserLogin {
    pub code_verifier: String,
    pub redirect_uri: String,
    pub callback: Option<oneshot::Receiver<Result<String, String>>>,
    pub callback_task: tokio::task::JoinHandle<()>,
}

/// Start the primary browser-based Microsoft login flow.
/// The callback listener is bound to an ephemeral loopback port and protected by OAuth state.
#[tauri::command]
pub async fn start_login(
    _app: AppHandle,
    state: State<'_, AppState>,
) -> Result<BrowserLoginInfo, AppError> {
    {
        let pending = state.auth.lock().await;
        if pending.is_some() {
            return Err(AppError::Validation(
                "A Microsoft sign-in is already in progress.".into(),
            ));
        }
    }
    {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if db::settings::get_setting(&db, "_device_code")?.is_some() {
            return Err(AppError::Validation(
                "A Microsoft device sign-in is already in progress.".into(),
            ));
        }
    }

    let std_listener = StdTcpListener::bind("127.0.0.1:0")
        .map_err(|e| AppError::Auth(format!("Unable to start local OAuth callback: {e}")))?;
    std_listener
        .set_nonblocking(true)
        .map_err(|e| AppError::Auth(format!("Unable to configure local OAuth callback: {e}")))?;
    let port = std_listener
        .local_addr()
        .map_err(|e| AppError::Auth(format!("Unable to read local OAuth callback port: {e}")))?
        .port();
    let listener = TcpListener::from_std(std_listener)
        .map_err(|e| AppError::Auth(format!("Unable to start local OAuth callback: {e}")))?;
    let (authorization_url, auth_state) = auth::start_auth_code_flow(port)
        .map_err(|e| AppError::Auth(format!("Unable to prepare Microsoft sign-in: {e}")))?;
    let expected_state = auth_state.state.clone();
    let (sender, callback) = oneshot::channel();

    let callback_task = tokio::spawn(async move {
        let result = auth::listen_for_auth_callback(listener, &expected_state).await;
        let _ = sender.send(result);
    });

    {
        let mut pending = state.auth.lock().await;
        if pending.is_some() {
            callback_task.abort();
            return Err(AppError::Validation(
                "A Microsoft sign-in is already in progress.".into(),
            ));
        }
        *pending = Some(PendingBrowserLogin {
            code_verifier: auth_state.code_verifier,
            redirect_uri: auth_state.redirect_uri.clone(),
            callback: Some(callback),
            callback_task,
        });
    }

    if let Err(error) = opener::open(&authorization_url) {
        let mut pending = state.auth.lock().await;
        if let Some(pending) = pending.take() {
            pending.callback_task.abort();
        }
        return Err(AppError::Auth(format!(
            "Unable to open the Microsoft sign-in page: {error}"
        )));
    }

    Ok(BrowserLoginInfo {
        authorization_url,
        redirect_uri: auth_state.redirect_uri,
    })
}

/// Complete the browser login after the loopback callback has been validated.
#[tauri::command]
pub async fn poll_login(state: State<'_, AppState>) -> Result<AccountInfo, AppError> {
    let (callback, code_verifier, redirect_uri) = {
        let mut pending = state.auth.lock().await;
        let pending = pending
            .as_mut()
            .ok_or_else(|| AppError::NotFound("No browser sign-in in progress".into()))?;
        (
            pending.callback.take().ok_or_else(|| {
                AppError::Auth("Microsoft sign-in is already being completed.".into())
            })?,
            pending.code_verifier.clone(),
            pending.redirect_uri.clone(),
        )
    };
    let result = callback
        .await
        .map_err(|_| AppError::Auth("Microsoft sign-in callback stopped unexpectedly.".into()))?;
    state.auth.lock().await.take();
    let code = result.map_err(AppError::Auth)?;
    let (msa_token, msa_refresh) = auth::exchange_code(&code, &code_verifier, &redirect_uri)
        .await
        .map_err(|e| AppError::Auth(format!("Microsoft token exchange failed: {e}")))?;

    complete_login(&state, &msa_token, &msa_refresh).await
}

/// Cancel either active Microsoft login method and clear temporary credentials.
#[tauri::command]
pub async fn cancel_login(state: State<'_, AppState>) -> Result<(), AppError> {
    let pending = {
        let mut pending = state.auth.lock().await;
        pending.take()
    };
    if let Some(pending) = pending {
        pending.callback_task.abort();
    }
    clear_device_login(&state)?;
    Ok(())
}

/// Start the device-code fallback when a browser callback cannot be used.
#[tauri::command]
pub async fn start_device_login(
    _app: AppHandle,
    state: State<'_, AppState>,
) -> Result<DeviceCodeInfo, AppError> {
    if state.auth.lock().await.is_some() {
        return Err(AppError::Validation(
            "A Microsoft sign-in is already in progress.".into(),
        ));
    }
    {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if db::settings::get_setting(&db, "_device_code")?.is_some() {
            return Err(AppError::Validation(
                "A Microsoft device sign-in is already in progress.".into(),
            ));
        }
    }
    let device_code = auth::start_device_code_flow()
        .await
        .map_err(|e| AppError::Auth(format!("Device code error: {e}")))?;

    let db = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::settings::set_setting(&db, "_device_code", &device_code.device_code)?;
    db::settings::set_setting(
        &db,
        "_device_code_expires",
        &device_code.expires_in.to_string(),
    )?;
    db::settings::set_setting(&db, "_device_code_started", &unix_timestamp().to_string())?;
    db::settings::set_setting(
        &db,
        "_device_code_interval",
        &device_code.interval.to_string(),
    )?;

    Ok(DeviceCodeInfo {
        user_code: device_code.user_code,
        verification_uri: device_code.verification_uri,
        message: device_code.message,
        interval: device_code.interval,
        expires_in: device_code.expires_in,
    })
}

/// Poll the device-code fallback until Microsoft returns tokens.
#[tauri::command]
pub async fn poll_device_login(state: State<'_, AppState>) -> Result<AccountInfo, AppError> {
    let (device_code, expired) = {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let code = db::settings::get_setting(&db, "_device_code")?
            .ok_or_else(|| AppError::NotFound("No device sign-in in progress".into()))?;
        let expires = db::settings::get_setting(&db, "_device_code_expires")?
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(900);
        let started = db::settings::get_setting(&db, "_device_code_started")?
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        (
            code,
            started > 0 && unix_timestamp().saturating_sub(started) >= expires,
        )
    };

    if expired {
        clear_device_login(&state)?;
        return Err(AppError::Auth(
            "The device sign-in code expired. Please start again.".into(),
        ));
    }

    let (msa_token, msa_refresh) = auth::poll_for_token(&device_code)
        .await
        .map_err(|e| AppError::Auth(e.to_string()))?;
    let account = complete_login(&state, &msa_token, &msa_refresh).await?;
    clear_device_login(&state)?;
    Ok(account)
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn clear_device_login(state: &AppState) -> Result<(), AppError> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    for key in [
        "_device_code",
        "_device_code_expires",
        "_device_code_started",
        "_device_code_interval",
    ] {
        let _ = db::settings::delete_setting(&db, key);
    }
    Ok(())
}

async fn complete_login(
    state: &AppState,
    msa_token: &str,
    msa_refresh: &str,
) -> Result<AccountInfo, AppError> {
    let (mc_token, _xuid) = auth::xbox_auth_chain(msa_token)
        .await
        .map_err(|e| AppError::Auth(e.to_string()))?;
    let profile = auth::get_minecraft_profile(&mc_token)
        .await
        .map_err(|e| AppError::Auth(e.to_string()))?;
    let skin_url = profile.skins.first().map(|skin| skin.url.clone());

    let db = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let account = db::accounts::Account {
        uuid: profile.id.clone(),
        username: profile.name.clone(),
        access_token: mc_token,
        refresh_token: msa_refresh.to_string(),
        skin_url,
    };
    db::accounts::upsert_account(&db, &account)?;

    Ok(AccountInfo {
        uuid: account.uuid,
        username: account.username,
        skin_url: account.skin_url,
    })
}

#[tauri::command]
pub fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountInfo>, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let accounts = db::accounts::get_all_accounts(&db)?;

    Ok(accounts
        .into_iter()
        .map(|account| AccountInfo {
            uuid: account.uuid,
            username: account.username,
            skin_url: account.skin_url,
        })
        .collect())
}

#[tauri::command]
pub fn remove_account(state: State<'_, AppState>, uuid: String) -> Result<(), AppError> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::accounts::delete_account(&db, &uuid)?;
    Ok(())
}

/// Refresh an account's Minecraft token using the stored MSA refresh token.
#[tauri::command]
pub async fn refresh_account_token(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<AccountInfo, AppError> {
    let refresh_token = {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let accounts = db::accounts::get_all_accounts(&db)?;
        let account = accounts
            .iter()
            .find(|account| account.uuid == uuid)
            .ok_or_else(|| AppError::NotFound("Account not found".into()))?;
        if account.refresh_token.is_empty() {
            return Err(AppError::Auth(
                "No refresh token stored. Please sign in again.".into(),
            ));
        }
        account.refresh_token.clone()
    };

    let (new_msa_token, new_msa_refresh) = auth::refresh_msa_token(&refresh_token)
        .await
        .map_err(|e| AppError::Auth(format!("MSA refresh failed: {e}")))?;
    let (mc_token, _xuid) = auth::xbox_auth_chain(&new_msa_token)
        .await
        .map_err(|e| AppError::Auth(format!("Xbox auth failed after refresh: {e}")))?;
    let profile = auth::get_minecraft_profile(&mc_token)
        .await
        .map_err(|e| AppError::Auth(format!("Profile fetch failed after refresh: {e}")))?;
    let skin_url = profile.skins.first().map(|skin| skin.url.clone());

    let db = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
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
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    db.execute(
        "UPDATE accounts SET last_used = ?1 WHERE uuid = ?2",
        rusqlite::params![chrono::Utc::now().to_rfc3339(), uuid],
    )?;

    let account = db::accounts::get_all_accounts(&db)?
        .into_iter()
        .find(|account| account.uuid == uuid)
        .ok_or_else(|| AppError::NotFound("Account not found".into()))?;

    Ok(AccountInfo {
        uuid: account.uuid,
        username: account.username,
        skin_url: account.skin_url,
    })
}
