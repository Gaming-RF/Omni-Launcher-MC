use serde::Serialize;

/// Info about a log file.
#[derive(Debug, Clone, Serialize)]
pub struct LogFileInfo {
    pub filename: String,
    pub size_bytes: u64,
    pub modified: String,
    pub log_type: String, // "latest", "debug", "crash", "server"
}

/// Cursor-based log reading result.
#[derive(Debug, Clone, Serialize)]
pub struct LogCursor {
    pub content: String,
    pub new_cursor: u64,
    pub has_more: bool,
}

/// List all log files for an instance.
#[tauri::command]
pub async fn get_log_files(instance_id: String) -> Result<Vec<LogFileInfo>, String> {
    let logs_dir = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("logs");

    if !logs_dir.exists() {
        return Ok(vec![]);
    }

    let mut files = Vec::new();
    let mut entries = tokio::fs::read_dir(&logs_dir)
        .await
        .map_err(|e| e.to_string())?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let metadata = entry.metadata().await.map_err(|e| e.to_string())?;
        let filename = entry.file_name().to_string_lossy().to_string();

        let log_type = if filename == "latest.log" {
            "latest".to_string()
        } else if filename.contains("debug") {
            "debug".to_string()
        } else if filename.contains("crash") {
            "crash".to_string()
        } else if filename.contains("server") {
            "server".to_string()
        } else {
            "other".to_string()
        };

        let modified = metadata
            .modified()
            .map(|t| {
                chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
            })
            .unwrap_or_default();

        files.push(LogFileInfo {
            filename,
            size_bytes: metadata.len(),
            modified,
            log_type,
        });
    }

    // Sort by modified time, newest first
    files.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(files)
}

/// Read log content from a cursor position (for live tailing).
#[tauri::command]
pub async fn read_log_cursor(
    instance_id: String,
    filename: String,
    cursor: u64,
    max_bytes: Option<u64>,
) -> Result<LogCursor, String> {
    let log_path = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("logs")
        .join(&filename);

    if !log_path.exists() {
        return Ok(LogCursor {
            content: String::new(),
            new_cursor: cursor,
            has_more: false,
        });
    }

    let data = tokio::fs::read(&log_path)
        .await
        .map_err(|e| e.to_string())?;

    let file_len = data.len() as u64;
    let max = max_bytes.unwrap_or(64 * 1024); // Default 64KB chunks

    if cursor >= file_len {
        return Ok(LogCursor {
            content: String::new(),
            new_cursor: cursor,
            has_more: false,
        });
    }

    let start = cursor as usize;
    let end = std::cmp::min(start + max as usize, data.len());

    let content = String::from_utf8_lossy(&data[start..end]).to_string();
    let new_cursor = end as u64;

    Ok(LogCursor {
        content,
        new_cursor,
        has_more: new_cursor < file_len,
    })
}

/// Read the full content of a log file.
#[tauri::command]
pub async fn read_log_file(
    instance_id: String,
    filename: String,
) -> Result<String, String> {
    let log_path = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("logs")
        .join(&filename);

    if !log_path.exists() {
        return Err("Log file not found".to_string());
    }

    tokio::fs::read_to_string(&log_path)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a specific log file.
#[tauri::command]
pub async fn delete_log_file(
    instance_id: String,
    filename: String,
) -> Result<(), String> {
    let log_path = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("logs")
        .join(&filename);

    if !log_path.exists() {
        return Err("Log file not found".to_string());
    }

    tokio::fs::remove_file(&log_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Delete all log files for an instance.
#[tauri::command]
pub async fn delete_all_logs(instance_id: String) -> Result<u32, String> {
    let logs_dir = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("logs");

    if !logs_dir.exists() {
        return Ok(0);
    }

    let mut count = 0u32;
    let mut entries = tokio::fs::read_dir(&logs_dir)
        .await
        .map_err(|e| e.to_string())?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        if entry.path().is_file() {
            tokio::fs::remove_file(entry.path())
                .await
                .map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    Ok(count)
}

/// Get log file size.
#[tauri::command]
pub async fn get_log_size(
    instance_id: String,
    filename: String,
) -> Result<u64, String> {
    let log_path = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("logs")
        .join(&filename);

    if !log_path.exists() {
        return Ok(0);
    }

    let metadata = tokio::fs::metadata(&log_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(metadata.len())
}
