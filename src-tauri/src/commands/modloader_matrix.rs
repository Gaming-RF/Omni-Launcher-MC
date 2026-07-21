use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct LoaderVersionInfo {
    pub version: String,
    pub stable: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct ModloaderMatrixEntry {
    pub loader: String,
    pub versions: Vec<LoaderVersionInfo>,
    pub latest_version: Option<String>,
    pub recommended_version: Option<String>,
    pub installed_version: Option<String>,
}

#[tauri::command]
pub async fn get_modloader_matrix(
    game_version: String,
) -> Result<Vec<ModloaderMatrixEntry>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let (fabric, forge, quilt, neoforge) = tokio::join!(
        fetch_fabric(&client, &game_version),
        fetch_forge(&client, &game_version),
        fetch_quilt(&client, &game_version),
        fetch_neoforge(&client, &game_version),
    );

    Ok(vec![
        fabric.unwrap_or_else(|_e| ModloaderMatrixEntry {
            loader: "fabric".into(),
            versions: vec![],
            latest_version: None,
            recommended_version: None,
            installed_version: None,
        }),
        forge.unwrap_or_else(|_e| ModloaderMatrixEntry {
            loader: "forge".into(),
            versions: vec![],
            latest_version: None,
            recommended_version: None,
            installed_version: None,
        }),
        quilt.unwrap_or_else(|_e| ModloaderMatrixEntry {
            loader: "quilt".into(),
            versions: vec![],
            latest_version: None,
            recommended_version: None,
            installed_version: None,
        }),
        neoforge.unwrap_or_else(|_e| ModloaderMatrixEntry {
            loader: "neoforge".into(),
            versions: vec![],
            latest_version: None,
            recommended_version: None,
            installed_version: None,
        }),
    ])
}

#[tauri::command]
pub async fn get_instance_modloader_matrix(
    state: tauri::State<'_, crate::AppState>,
    instance_id: String,
) -> Result<Vec<ModloaderMatrixEntry>, String> {
    let (game_version, installed_loader, installed_loader_version) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let instance = crate::db::instances::get_instance(&db, &instance_id)
            .map_err(|e| e.to_string())?
            .ok_or("Instance not found")?;
        (
            instance.game_version,
            instance.loader.clone(),
            instance.loader_version.clone(),
        )
    };

    let mut matrix = get_modloader_matrix(game_version).await?;

    // Mark installed version
    for entry in &mut matrix {
        if entry.loader == installed_loader {
            entry.installed_version = installed_loader_version.clone();
        }
    }

    Ok(matrix)
}

async fn fetch_fabric(
    client: &reqwest::Client,
    game_version: &str,
) -> Result<ModloaderMatrixEntry, String> {
    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}",
        game_version
    );
    let resp: Vec<serde_json::Value> = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let versions: Vec<LoaderVersionInfo> = resp
        .iter()
        .take(20)
        .filter_map(|v| {
            let version = v.get("loader")?.get("version")?.as_str()?.to_string();
            let stable = v
                .get("loader")
                .and_then(|l| l.get("stable"))
                .and_then(|s| s.as_bool())
                .unwrap_or(false);
            Some(LoaderVersionInfo { version, stable })
        })
        .collect();

    let latest = versions.first().map(|v| v.version.clone());
    let recommended = versions.iter().find(|v| v.stable).map(|v| v.version.clone());

    Ok(ModloaderMatrixEntry {
        loader: "fabric".into(),
        versions,
        latest_version: latest,
        recommended_version: recommended,
        installed_version: None,
    })
}

async fn fetch_forge(
    client: &reqwest::Client,
    game_version: &str,
) -> Result<ModloaderMatrixEntry, String> {
    // Forge versions from maven metadata
    let url = "https://files.minecraftforge.net/maven/net/minecraftforge/forge/maven-metadata.xml";
    let body = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    // Parse versions that match our game version
    let prefix = format!("{}-", game_version);
    let mut versions: Vec<LoaderVersionInfo> = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
            let ver = trimmed
                .trim_start_matches("<version>")
                .trim_end_matches("</version>");
            if ver.starts_with(&prefix) {
                let loader_ver = ver.strip_prefix(&prefix).unwrap_or(ver);
                versions.push(LoaderVersionInfo {
                    version: loader_ver.to_string(),
                    stable: true,
                });
            }
        }
    }
    versions.reverse(); // newest first
    versions.truncate(20);

    let latest = versions.first().map(|v| v.version.clone());

    Ok(ModloaderMatrixEntry {
        loader: "forge".into(),
        versions,
        latest_version: latest.clone(),
        recommended_version: latest,
        installed_version: None,
    })
}

async fn fetch_quilt(
    client: &reqwest::Client,
    game_version: &str,
) -> Result<ModloaderMatrixEntry, String> {
    let url = format!(
        "https://meta.quiltmc.org/v3/versions/loader/{}",
        game_version
    );
    let resp: Vec<serde_json::Value> = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let versions: Vec<LoaderVersionInfo> = resp
        .iter()
        .take(20)
        .filter_map(|v| {
            let version = v.get("loader")?.get("version")?.as_str()?.to_string();
            let is_release = v
                .get("loader")
                .and_then(|l| l.get("version"))
                .and_then(|_| Some(true)) // Quilt versions are generally stable
                .unwrap_or(true);
            Some(LoaderVersionInfo {
                version,
                stable: is_release,
            })
        })
        .collect();

    let latest = versions.first().map(|v| v.version.clone());

    Ok(ModloaderMatrixEntry {
        loader: "quilt".into(),
        versions,
        latest_version: latest.clone(),
        recommended_version: latest,
        installed_version: None,
    })
}

async fn fetch_neoforge(
    client: &reqwest::Client,
    game_version: &str,
) -> Result<ModloaderMatrixEntry, String> {
    let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
    let body = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    // NeoForge versions are like 21.0.1-beta for MC 1.21
    let mc_major = game_version.split('.').nth(1).unwrap_or("21");
    let prefix = format!("{}.", mc_major);

    let mut versions: Vec<LoaderVersionInfo> = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
            let ver = trimmed
                .trim_start_matches("<version>")
                .trim_end_matches("</version>");
            if ver.starts_with(&prefix) {
                let stable = !ver.contains("beta") && !ver.contains("alpha");
                versions.push(LoaderVersionInfo {
                    version: ver.to_string(),
                    stable,
                });
            }
        }
    }
    versions.reverse(); // newest first
    versions.truncate(20);

    let latest = versions.first().map(|v| v.version.clone());
    let recommended = versions.iter().find(|v| v.stable).map(|v| v.version.clone());

    Ok(ModloaderMatrixEntry {
        loader: "neoforge".into(),
        versions,
        latest_version: latest,
        recommended_version: recommended,
        installed_version: None,
    })
}
