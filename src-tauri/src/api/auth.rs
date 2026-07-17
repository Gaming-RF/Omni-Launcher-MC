use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Microsoft OAuth2 Device Code Flow constants
// IMPORTANT: Replace with your own Azure AD app registration client ID
const CLIENT_ID: &str = "YOUR_CLIENT_ID_HERE";
const DEVICE_CODE_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const SCOPE: &str = "XboxLive.signin offline_access";

// Xbox Live endpoints
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

// Minecraft services
const MC_AUTH_URL: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub uuid: String,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub skin_url: Option<String>,
}

/// Start the Microsoft Device Code authentication flow.
/// Returns the device code info for the user to complete in their browser.
pub async fn start_device_code_flow() -> Result<DeviceCodeResponse> {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("scope", SCOPE);

    let resp = client
        .post(DEVICE_CODE_URL)
        .form(&params)
        .send()
        .await
        .context("Failed to request device code")?;

    let body: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse device code response")?;

    if let Some(error) = body.get("error") {
        anyhow::bail!(
            "Device code error: {}",
            error.as_str().unwrap_or("unknown")
        );
    }

    Ok(DeviceCodeResponse {
        device_code: body["device_code"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        user_code: body["user_code"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        verification_uri: body["verification_uri"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        expires_in: body["expires_in"].as_u64().unwrap_or(900) as u32,
        interval: body["interval"].as_u64().unwrap_or(5) as u32,
        message: body["message"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
    })
}

/// Poll the token endpoint. Returns Ok(tokens) when the user completes auth,
/// or Err("authorization_pending") if still waiting.
pub async fn poll_for_token(device_code: &str) -> Result<(String, String)> {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");
    params.insert("device_code", device_code);

    let resp = client.post(TOKEN_URL).form(&params).send().await?;
    let token_resp: TokenResponse = resp.json().await?;

    if let Some(error) = &token_resp.error {
        anyhow::bail!(error.clone());
    }

    let access = token_resp
        .access_token
        .context("No access_token in response")?;
    let refresh = token_resp
        .refresh_token
        .context("No refresh_token in response")?;

    Ok((access, refresh))
}

/// Complete the full auth chain: MSA token -> Xbox Live -> XSTS -> Minecraft.
/// Returns (mc_access_token, xuid).
pub async fn xbox_auth_chain(msa_token: &str) -> Result<(String, String)> {
    let client = reqwest::Client::new();

    // Step 1: Xbox Live authentication
    let xbl_body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", msa_token)
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "Service": "JWT"
    });

    let xbl_resp: serde_json::Value = client
        .post(XBL_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&xbl_body)
        .send()
        .await?
        .json()
        .await?;

    let xbl_token = xbl_resp["Token"]
        .as_str()
        .context("No Token in Xbox Live response")?;
    let uhs = xbl_resp["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .context("No uhs in Xbox Live response")?;

    // Step 2: XSTS authorization
    let xsts_body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "Service": "JWT"
    });

    let xsts_resp: serde_json::Value = client
        .post(XSTS_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&xsts_body)
        .send()
        .await?
        .json()
        .await?;

    if let Some(err_code) = xsts_resp["XErr"].as_i64() {
        if err_code != 0 {
            anyhow::bail!("XSTS error code: {}", err_code);
        }
    }

    let xsts_token = xsts_resp["Token"]
        .as_str()
        .context("No Token in XSTS response")?;

    // Step 3: Minecraft authentication
    let mc_body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", uhs, xsts_token)
    });

    let mc_resp: serde_json::Value = client
        .post(MC_AUTH_URL)
        .header("Content-Type", "application/json")
        .json(&mc_body)
        .send()
        .await?
        .json()
        .await?;

    let mc_token = mc_resp["access_token"]
        .as_str()
        .context("No access_token in MC auth response")?;

    Ok((mc_token.to_string(), uhs.to_string()))
}

/// Fetch the Minecraft profile (username, UUID, skins).
pub async fn get_minecraft_profile(mc_token: &str) -> Result<MinecraftProfile> {
    let client = reqwest::Client::new();

    let profile: MinecraftProfile = client
        .get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", mc_token))
        .send()
        .await?
        .json()
        .await?;

    Ok(profile)
}
