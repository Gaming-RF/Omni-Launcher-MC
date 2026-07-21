use serde::Serialize;
use std::path::PathBuf;

/// Result of creating a desktop shortcut.
#[derive(Serialize)]
pub struct ShortcutResult {
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Create a desktop shortcut for launching an instance directly.
#[allow(clippy::needless_return)]
#[tauri::command]
pub async fn create_desktop_shortcut(
    _app_handle: tauri::AppHandle,
    instance_id: String,
    instance_name: String,
    output_dir: Option<String>,
    server_address: Option<String>,
) -> Result<ShortcutResult, String> {
    let out_dir = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::desktop_dir().unwrap_or_else(|| PathBuf::from(".")));

    // Build the deep link URL
    let mut launch_url = format!("omnilauncher://instance/{}", instance_id);
    if let Some(ref server) = server_address {
        launch_url.push_str(&format!("?server={}", urlencoding::encode(server)));
    }

    let safe_name = instance_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();

    #[cfg(target_os = "linux")]
    {
        let desktop_file = out_dir.join(format!("{}-OmniLauncher.desktop", safe_name));
        let exec_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "omnilauncher-mc".to_string());

        let content = format!(
            r#"[Desktop Entry]
Name={instance_name}
Comment=Launch {instance_name} via OmniLauncherMC
Exec={exec_path} --instance {instance_id}
Icon=minecraft
Terminal=false
Type=Application
Categories=Game;
"#
        );

        tokio::fs::write(&desktop_file, content)
            .await
            .map_err(|e| e.to_string())?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&desktop_file)
                .await
                .map_err(|e| e.to_string())?
                .permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&desktop_file, perms)
                .await
                .map_err(|e| e.to_string())?;
        }

        return Ok(ShortcutResult {
            path: desktop_file.to_string_lossy().to_string(),
            success: true,
            error: None,
        });
    }

    #[cfg(target_os = "windows")]
    {
        let lnk_file = out_dir.join(format!("{}.lnk", instance_name));
        // On Windows we'd use a .lnk shortcut, but for simplicity create a .bat
        let bat_file = out_dir
            .join(format!("{}-OmniLauncher.bat", safe_name));
        let content = format!(
            "@echo off\r\nstart \"\" \"{exe}\" --instance {id}\r\n",
            exe = std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            id = instance_id
        );

        tokio::fs::write(&bat_file, content)
            .await
            .map_err(|e| e.to_string())?;

        return Ok(ShortcutResult {
            path: bat_file.to_string_lossy().to_string(),
            success: true,
            error: None,
        });
    }

    #[cfg(target_os = "macos")]
    {
        let app_dir = out_dir.join(format!("{}.command", safe_name));
        let content = format!(
            "#!/bin/bash\nopen -a '{}' --args --instance {}\n",
            std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            instance_id
        );

        tokio::fs::write(&app_dir, &content)
            .await
            .map_err(|e| e.to_string())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&app_dir)
                .await
                .map_err(|e| e.to_string())?
                .permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&app_dir, perms)
                .await
                .map_err(|e| e.to_string())?;
        }

        return Ok(ShortcutResult {
            path: app_dir.to_string_lossy().to_string(),
            success: true,
            error: None,
        });
    }
}

/// Get the default shortcut output directory (Desktop).
#[tauri::command]
pub fn get_shortcut_default_dir() -> Result<String, String> {
    Ok(dirs::desktop_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .to_string_lossy()
        .to_string())
}
