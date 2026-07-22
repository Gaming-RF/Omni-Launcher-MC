use crate::error::AppError;
use serde::Serialize;

/// A Minecraft skin with metadata.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SkinInfo {
    pub texture_url: Option<String>,
    pub variant: String, // "slim" or "classic"
    pub cape_url: Option<String>,
}

/// Upload and equip a custom skin.
#[tauri::command]
pub async fn upload_skin(
    _account_uuid: String,
    _skin_data: Vec<u8>,
    _variant: String,
) -> Result<SkinInfo, AppError> {
    // This requires Mojang API authentication
    // POST https://api.minecraftservices.com/minecraft/profile/skins
    // with multipart form data (file + variant)
    Err(AppError::Internal("Skin upload requires Microsoft authentication. Please sign in first.".to_string()))
}

/// Get the current skin info for an account.
#[tauri::command]
pub async fn get_skin_info(account_uuid: String) -> Result<SkinInfo, AppError> {
    // Fetch from Mojang session API
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}",
        account_uuid
    );

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!("Failed to fetch profile: {}", resp.status())));
    }

    let body: serde_json::Value = resp.json().await?;

    let mut skin_url = None;
    let mut variant = "classic".to_string();
    let mut cape_url = None;

    if let Some(properties) = body["properties"].as_array() {
        for prop in properties {
            if prop["name"].as_str() == Some("textures") {
                if let Some(value_str) = prop["value"].as_str() {
                    // Base64 decode the textures value
                    use base64::Engine;
                    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(value_str)
                    {
                        if let Ok(textures) = serde_json::from_slice::<serde_json::Value>(&decoded)
                        {
                            if let Some(skin) = textures["textures"]["SKIN"].as_object() {
                                skin_url = skin["url"].as_str().map(|s| s.to_string());
                                if let Some(meta) = skin["metadata"].as_object() {
                                    if meta["model"].as_str() == Some("slim") {
                                        variant = "slim".to_string();
                                    }
                                }
                            }
                            if let Some(cape) = textures["textures"]["CAPE"].as_object() {
                                cape_url = cape["url"].as_str().map(|s| s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(SkinInfo {
        texture_url: skin_url,
        variant,
        cape_url,
    })
}

/// Reset to the default Steve/Alex skin.
#[tauri::command]
pub async fn reset_skin(_account_uuid: String) -> Result<(), AppError> {
    // DELETE https://api.minecraftservices.com/minecraft/profile/skins
    Err(AppError::Internal("Skin reset requires Microsoft authentication.".to_string()))
}

/// Get available capes for an account.
#[tauri::command]
pub async fn get_capes(_account_uuid: String) -> Result<Vec<CapeInfo>, AppError> {
    // GET https://api.minecraftservices.com/minecraft/profile/capes
    Ok(vec![])
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct CapeInfo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub state: String, // "INACTIVE" or "ACTIVE"
}
