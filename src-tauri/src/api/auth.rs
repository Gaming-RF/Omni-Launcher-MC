use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};

// Microsoft OAuth2 - registered by PrismLauncher community
// This is a public client (no secret needed) registered on Microsoft Identity Platform
const CLIENT_ID: &str = "c36a9fb6-4f2a-41ff-90bd-ae7cc92031eb";
const REDIRECT_URI: &str = "http://localhost:12749/auth/callback";

// Microsoft endpoints (consumers tenant = personal accounts only)
const AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const SCOPE: &str = "XboxLive.signin offline_access";

// Xbox Live endpoints
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

// Minecraft services
const MC_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthCodeState {
    pub code_verifier: String,
    pub state: String,
    pub port: u16,
}

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

/// Generate a random string for PKCE/state
fn random_string(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)[..len].to_string()
}

/// SHA256 hash for PKCE challenge
fn sha256(data: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    sha2::Sha256::digest(data).to_vec()
}

/// Start the OAuth2 auth code flow with PKCE.
/// Returns the authorization URL to open in the browser and the state needed for the callback.
pub fn start_auth_code_flow() -> Result<(String, AuthCodeState)> {
    let code_verifier = random_string(64);
    let state = random_string(32);

    // PKCE challenge = base64url(sha256(verifier))
    let challenge =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sha256(code_verifier.as_bytes()));

    let auth_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256&response_mode=query",
        AUTHORIZE_URL,
        CLIENT_ID,
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(SCOPE),
        state,
        challenge,
    );

    // Parse port from redirect URI
    let port = 12749u16;

    Ok((
        auth_url,
        AuthCodeState {
            code_verifier,
            state,
            port,
        },
    ))
}

/// Exchange an authorization code for tokens.
pub async fn exchange_code(code: &str, code_verifier: &str) -> Result<(String, String)> {
    let client = reqwest::Client::new();

    let mut params = std::collections::HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("code", code);
    params.insert("redirect_uri", REDIRECT_URI);
    params.insert("grant_type", "authorization_code");
    params.insert("code_verifier", code_verifier);

    let resp = client.post(TOKEN_URL).form(&params).send().await?;

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    let token_resp: TokenResponse = serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("Failed to decode token response ({}): {} — body: {}", status, e, &body_text[..body_text.len().min(500)]))?;

    if let Some(error) = &token_resp.error {
        anyhow::bail!(
            "Token error: {} - {}",
            error,
            token_resp.error_description.as_deref().unwrap_or("")
        );
    }

    let access = token_resp
        .access_token
        .context("No access_token in response")?;
    let refresh = token_resp
        .refresh_token
        .context("No refresh_token in response")?;

    Ok((access, refresh))
}

/// Refresh an expired MSA token using the refresh token.
pub async fn refresh_msa_token(refresh_token: &str) -> Result<(String, String)> {
    let client = reqwest::Client::new();

    let mut params = std::collections::HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("refresh_token", refresh_token);
    params.insert("grant_type", "refresh_token");

    let resp = client.post(TOKEN_URL).form(&params).send().await?;
    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    let token_resp: TokenResponse = serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("Failed to decode refresh response ({}): {} — body: {}", status, e, &body_text[..body_text.len().min(500)]))?;

    if let Some(error) = &token_resp.error {
        anyhow::bail!("Refresh error: {}", error);
    }

    let access = token_resp
        .access_token
        .context("No access_token in refresh response")?;
    let refresh = token_resp
        .refresh_token
        .unwrap_or_else(|| refresh_token.to_string());

    Ok((access, refresh))
}

/// Start the Microsoft Device Code authentication flow.
/// Returns the device code info for the user to complete in their browser.
pub async fn start_device_code_flow() -> Result<DeviceCodeResponse> {
    let client = reqwest::Client::new();

    let mut params = std::collections::HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("scope", SCOPE);

    let resp = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
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
            "Device code error: {} - {}",
            error.as_str().unwrap_or("unknown"),
            body.get("error_description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
        );
    }

    Ok(DeviceCodeResponse {
        device_code: body["device_code"].as_str().unwrap_or_default().to_string(),
        user_code: body["user_code"].as_str().unwrap_or_default().to_string(),
        verification_uri: body["verification_uri"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        expires_in: body["expires_in"].as_u64().unwrap_or(900) as u32,
        interval: body["interval"].as_u64().unwrap_or(5) as u32,
        message: body["message"].as_str().unwrap_or_default().to_string(),
    })
}

/// Poll the token endpoint. Returns Ok(tokens) when the user completes auth,
/// or Err("authorization_pending") if still waiting.
pub async fn poll_for_token(device_code: &str) -> Result<(String, String)> {
    let client = reqwest::Client::new();

    let mut params = std::collections::HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");
    params.insert("device_code", device_code);

    let resp = client.post(TOKEN_URL).form(&params).send().await?;
    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    let token_resp: TokenResponse = serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("Failed to decode poll response ({}): {} — body: {}", status, e, &body_text[..body_text.len().min(500)]))?;

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

    let xbl_resp_raw = client
        .post(XBL_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("x-xbl-contract-version", "1")
        .json(&xbl_body)
        .send()
        .await
        .context("Failed to send Xbox Live request")?;
    let xbl_status = xbl_resp_raw.status();
    let xbl_text = xbl_resp_raw.text().await.unwrap_or_default();
    if xbl_text.trim().is_empty() {
        anyhow::bail!(
            "Xbox Live returned {} with an empty body. This usually means the Microsoft token is \
             invalid or missing the XboxLive.signin scope. Try signing in again.",
            xbl_status
        );
    }
    let xbl_resp: serde_json::Value = serde_json::from_str(&xbl_text)
        .map_err(|e| anyhow::anyhow!("Xbox Live decode error ({}): {} — body: {}", xbl_status, e, &xbl_text[..xbl_text.len().min(500)]))?;

    if let Some(err) = xbl_resp.get("error") {
        let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
        let msg = err.get("message").and_then(|m| m.as_str()).unwrap_or("");
        anyhow::bail!("Xbox Live error (code {}): {}", code, msg);
    }

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

    let xsts_resp_raw = client
        .post(XSTS_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&xsts_body)
        .send()
        .await
        .context("Failed to send XSTS request")?;
    let xsts_status = xsts_resp_raw.status();
    let xsts_text = xsts_resp_raw.text().await.unwrap_or_default();
    if xsts_text.trim().is_empty() {
        anyhow::bail!("XSTS returned {} with an empty body.", xsts_status);
    }
    let xsts_resp: serde_json::Value = serde_json::from_str(&xsts_text)
        .map_err(|e| anyhow::anyhow!("XSTS decode error ({}): {} — body: {}", xsts_status, e, &xsts_text[..xsts_text.len().min(500)]))?;

    if let Some(err_code) = xsts_resp["XErr"].as_i64() {
        if err_code != 0 {
            let msg = match err_code {
                2148916233 => "This Microsoft account does not have an Xbox account.",
                2148916235 => {
                    "This Xbox account is from a country/region where Xbox Live is not available."
                }
                2148916236 => "This Xbox account needs parental approval.",
                2148916237 => "This Xbox account is banned.",
                2148916238 => "This Microsoft account needs to complete adult verification.",
                _ => "Xbox authentication failed.",
            };
            anyhow::bail!("XSTS error ({}): {}", err_code, msg);
        }
    }

    let xsts_token = xsts_resp["Token"]
        .as_str()
        .context("No Token in XSTS response")?;

    // Step 3: Minecraft authentication
    let mc_body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", uhs, xsts_token)
    });

    let mc_resp_raw = client
        .post(MC_AUTH_URL)
        .header("Content-Type", "application/json")
        .json(&mc_body)
        .send()
        .await
        .context("Failed to send MC auth request")?;
    let mc_status = mc_resp_raw.status();
    let mc_text = mc_resp_raw.text().await.unwrap_or_default();
    if mc_text.trim().is_empty() {
        anyhow::bail!("MC auth returned {} with an empty body.", mc_status);
    }
    let mc_resp: serde_json::Value = serde_json::from_str(&mc_text)
        .map_err(|e| anyhow::anyhow!("MC auth decode error ({}): {} — body: {}", mc_status, e, &mc_text[..mc_text.len().min(500)]))?;

    let mc_token = mc_resp["access_token"]
        .as_str()
        .context("No access_token in MC auth response")?;

    Ok((mc_token.to_string(), uhs.to_string()))
}

/// Fetch the Minecraft profile (username, UUID, skins).
pub async fn get_minecraft_profile(mc_token: &str) -> Result<MinecraftProfile> {
    let client = reqwest::Client::new();

    let resp = client
        .get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", mc_token))
        .send()
        .await?;

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        anyhow::bail!("Profile fetch failed ({}): {}", status, body_text);
    }

    let profile: MinecraftProfile = serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("Profile decode error ({}): {} — body: {}", status, e, &body_text[..body_text.len().min(500)]))?;
    Ok(profile)
}
