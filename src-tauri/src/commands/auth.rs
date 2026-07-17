use serde::{Deserialize, Serialize};
use tauri::command;

/// Response from Microsoft Device Code flow initiation
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
    pub message: String,
}

/// Minecraft profile returned after successful auth
#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub skins: Vec<Skin>,
    pub capes: Vec<Cape>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cape {
    pub id: String,
    pub state: String,
    pub url: String,
}

/// Stored account info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub uuid: String,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub skin_url: Option<String>,
}

/// Starts Microsoft Device Code authentication flow.
/// Returns the device code + user code for the user to enter.
#[command]
pub async fn login_start() -> Result<DeviceCodeResponse, String> {
    // TODO: Implement Microsoft OAuth2 Device Code flow
    // 1. POST https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode
    //    client_id=<MSA_CLIENT_ID>&scope=XboxLive.signin%20offline_access
    // 2. Return device_code, user_code, verification_uri, message
    Err("Not yet implemented".to_string())
}

/// Polls the Microsoft token endpoint until the user completes auth.
/// On success, chains through Xbox Live -> XSTS -> Minecraft auth.
#[command]
pub async fn login_poll(device_code: String) -> Result<Account, String> {
    // TODO: Implement token polling and auth chain
    // 1. Poll POST https://login.microsoftonline.com/consumers/oauth2/v2.0/token
    //    until user completes auth (handle "authorization_pending" gracefully)
    // 2. Xbox Live: POST https://user.auth.xboxlive.com/user/authenticate
    // 3. XSTS: POST https://xsts.auth.xboxlive.com/xsts/authorize
    // 4. MC auth: POST https://api.minecraftservices.com/authentication/login_with_xbox
    // 5. Profile: GET https://api.minecraftservices.com/minecraft/profile
    // 6. Store account in DB
    Err("Not yet implemented".to_string())
}

/// Returns the current Minecraft profile for the active account.
#[command]
pub async fn get_profile() -> Result<MinecraftProfile, String> {
    // TODO: Fetch from https://api.minecraftservices.com/minecraft/profile
    // using stored access token
    Err("Not yet implemented".to_string())
}

/// Logs out the active account, removing stored tokens.
#[command]
pub async fn logout(uuid: String) -> Result<(), String> {
    // TODO: Remove account from DB, clear stored tokens
    Err("Not yet implemented".to_string())
}

/// Lists all stored accounts.
#[command]
pub async fn list_accounts() -> Result<Vec<Account>, String> {
    // TODO: Query accounts from DB
    Err("Not yet implemented".to_string())
}
