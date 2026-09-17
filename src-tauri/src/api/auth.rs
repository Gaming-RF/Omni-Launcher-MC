use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::timeout;
use url::Url;

// Microsoft OAuth2 - registered by PrismLauncher community
// This is a public client (no secret needed) registered on Microsoft Identity Platform
const CLIENT_ID: &str = "c36a9fb6-4f2a-41ff-90bd-ae7cc92031eb";
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
    pub redirect_uri: String,
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
pub fn start_auth_code_flow(port: u16) -> Result<(String, AuthCodeState)> {
    let code_verifier = random_string(64);
    let state = random_string(32);
    let redirect_uri = format!("http://127.0.0.1:{port}/auth/callback");

    // PKCE challenge = base64url(sha256(verifier))
    let challenge =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sha256(code_verifier.as_bytes()));

    let mut query = url::form_urlencoded::Serializer::new(String::new());
    query.append_pair("client_id", CLIENT_ID);
    query.append_pair("response_type", "code");
    query.append_pair("redirect_uri", &redirect_uri);
    query.append_pair("scope", SCOPE);
    query.append_pair("state", &state);
    query.append_pair("code_challenge", &challenge);
    query.append_pair("code_challenge_method", "S256");
    query.append_pair("response_mode", "query");
    let auth_url = format!("{AUTHORIZE_URL}?{}", query.finish());

    Ok((
        auth_url,
        AuthCodeState {
            code_verifier,
            state,
            redirect_uri,
        },
    ))
}

struct CallbackRequest {
    state: String,
    code: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

fn parse_callback_target(target: &str) -> Result<CallbackRequest, String> {
    let url = Url::parse(&format!("http://127.0.0.1{target}"))
        .map_err(|_| "Invalid callback request".to_string())?;
    if url.path() != "/auth/callback" {
        return Err("Invalid callback path".to_string());
    }

    let mut state = None;
    let mut code = None;
    let mut error = None;
    let mut error_description = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "state" => state = Some(value.into_owned()),
            "code" => code = Some(value.into_owned()),
            "error" => error = Some(value.into_owned()),
            "error_description" => error_description = Some(value.into_owned()),
            _ => {}
        }
    }

    Ok(CallbackRequest {
        state: state.ok_or_else(|| "Missing callback state".to_string())?,
        code,
        error,
        error_description,
    })
}

async fn callback_response(mut stream: tokio::net::TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
}

/// Wait for a validated OAuth callback on a loopback listener.
pub async fn listen_for_auth_callback(
    listener: TcpListener,
    expected_state: &str,
) -> Result<String, String> {
    timeout(Duration::from_secs(300), async {
        loop {
            let (mut stream, peer) = listener
                .accept()
                .await
                .map_err(|_| "Unable to accept OAuth callback".to_string())?;
            if !peer.ip().is_loopback() {
                callback_response(stream, "403 Forbidden", "This callback is local only.").await;
                continue;
            }

            let mut request = [0u8; 8192];
            let size = stream
                .read(&mut request)
                .await
                .map_err(|_| "Unable to read OAuth callback".to_string())?;
            let request_line = std::str::from_utf8(&request[..size])
                .ok()
                .and_then(|request| request.lines().next())
                .unwrap_or_default();
            let target = request_line
                .strip_prefix("GET ")
                .and_then(|request| request.split_once(" HTTP/").map(|(target, _)| target));

            let Some(target) = target else {
                callback_response(stream, "400 Bad Request", "Invalid OAuth callback.").await;
                continue;
            };
            let parsed = match parse_callback_target(target) {
                Ok(parsed) => parsed,
                Err(_) => {
                    callback_response(stream, "400 Bad Request", "Invalid OAuth callback.").await;
                    continue;
                }
            };
            if parsed.state != expected_state {
                callback_response(stream, "400 Bad Request", "Invalid OAuth callback state.").await;
                continue;
            }

            if let Some(error) = parsed.error {
                callback_response(
                    stream,
                    "400 Bad Request",
                    "Microsoft sign-in was cancelled or denied. You can close this window.",
                )
                .await;
                return Err(format!(
                    "Microsoft sign-in failed: {error}{}",
                    parsed
                        .error_description
                        .map(|description| format!(" ({description})"))
                        .unwrap_or_default()
                ));
            }

            let Some(code) = parsed.code else {
                callback_response(stream, "400 Bad Request", "Missing authorization code.").await;
                return Err("Microsoft callback did not contain an authorization code".to_string());
            };
            callback_response(
                stream,
                "200 OK",
                "Microsoft sign-in complete. You can return to OmniLauncherMC.",
            )
            .await;
            return Ok(code);
        }
    })
    .await
    .map_err(|_| "Microsoft sign-in timed out. Please try again.".to_string())?
}

/// Exchange an authorization code for tokens.
pub async fn exchange_code(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<(String, String)> {
    let client = reqwest::Client::new();

    let mut params = std::collections::HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("code", code);
    params.insert("redirect_uri", redirect_uri);
    params.insert("grant_type", "authorization_code");
    params.insert("code_verifier", code_verifier);

    let resp = client.post(TOKEN_URL).form(&params).send().await?;

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    let token_resp: TokenResponse = serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("Failed to decode token response ({status}): {e}"))?;

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
        .map_err(|e| anyhow::anyhow!("Failed to decode refresh response ({status}): {e}"))?;

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
        device_code: body
            .get("device_code")
            .and_then(|value| value.as_str())
            .context("Device code response did not include a device code")?
            .to_string(),
        user_code: body
            .get("user_code")
            .and_then(|value| value.as_str())
            .context("Device code response did not include a user code")?
            .to_string(),
        verification_uri: body
            .get("verification_uri")
            .or_else(|| body.get("verification_uri_complete"))
            .and_then(|value| value.as_str())
            .context("Device code response did not include a verification URI")?
            .to_string(),
        expires_in: body
            .get("expires_in")
            .and_then(|value| value.as_u64())
            .context("Device code response did not include an expiry")? as u32,
        interval: body
            .get("interval")
            .and_then(|value| value.as_u64())
            .unwrap_or(5) as u32,
        message: body
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
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
    params.insert("scope", SCOPE);

    let resp = client.post(TOKEN_URL).form(&params).send().await?;
    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    let token_resp: TokenResponse = serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("Failed to decode poll response ({status}): {e}"))?;

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
        .map_err(|e| anyhow::anyhow!("Xbox Live decode error ({xbl_status}): {e}"))?;

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
        .map_err(|e| anyhow::anyhow!("XSTS decode error ({xsts_status}): {e}"))?;

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
        .map_err(|e| anyhow::anyhow!("MC auth decode error ({mc_status}): {e}"))?;

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
        anyhow::bail!("Profile fetch failed ({status})");
    }

    let profile: MinecraftProfile = serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("Profile decode error ({status}): {e}"))?;
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_url_uses_loopback_and_pkce() {
        let (url, state) = start_auth_code_flow(45678).expect("auth flow");
        let parsed = Url::parse(&url).expect("authorization URL");
        let query: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();
        assert_eq!(query.get("redirect_uri"), Some(&state.redirect_uri));
        assert_eq!(query.get("state"), Some(&state.state));
        assert_eq!(
            query.get("code_challenge_method"),
            Some(&"S256".to_string())
        );
        assert!(state.redirect_uri.starts_with("http://127.0.0.1:45678/"));
        assert_ne!(query.get("code_challenge"), Some(&state.code_verifier));
    }

    #[test]
    fn callback_parser_requires_expected_shape() {
        let callback =
            parse_callback_target("/auth/callback?code=abc%2B123&state=expected&ignored=value")
                .expect("callback");
        assert_eq!(callback.state, "expected");
        assert_eq!(callback.code.as_deref(), Some("abc+123"));
        assert!(parse_callback_target("/other?code=abc&state=expected").is_err());
        assert!(parse_callback_target("/auth/callback?code=abc").is_err());
    }
}
