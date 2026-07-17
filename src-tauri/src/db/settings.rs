use anyhow::Result;

/// Get a setting value by key.
pub fn get_setting(db: &rusqlite::Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = db.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query_map([key], |row| row.get::<_, String>(0))?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Set a setting value (upsert).
pub fn set_setting(db: &rusqlite::Connection, key: &str, value: &str) -> Result<()> {
    db.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

/// Delete a setting.
pub fn delete_setting(db: &rusqlite::Connection, key: &str) -> Result<()> {
    db.execute("DELETE FROM settings WHERE key = ?1", [key])?;
    Ok(())
}

/// Get the CurseForge API key from settings.
pub fn get_curseforge_api_key(db: &rusqlite::Connection) -> Result<Option<String>> {
    get_setting(db, "curseforge_api_key")
}

/// Get the allocated memory default from settings (in MB).
pub fn get_default_memory(db: &rusqlite::Connection) -> Result<i64> {
    match get_setting(db, "default_memory_mb")? {
        Some(val) => Ok(val.parse().unwrap_or(4096)),
        None => Ok(4096), // Default 4GB
    }
}

/// Get the Java path from settings.
pub fn get_java_path(db: &rusqlite::Connection) -> Result<Option<String>> {
    get_setting(db, "java_path")
}
