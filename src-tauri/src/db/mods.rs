use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledMod {
    pub id: i64,
    pub instance_id: String,
    pub mod_id: String,
    pub source: String,
    pub name: String,
    pub version: String,
    pub file_name: String,
    pub enabled: bool,
    pub installed_at: String,
}

/// Get all mods for an instance.
pub fn get_instance_mods(
    db: &rusqlite::Connection,
    instance_id: &str,
) -> Result<Vec<InstalledMod>> {
    let mut stmt = db.prepare(
        "SELECT id, instance_id, mod_id, source, name, version, file_name, enabled, installed_at 
         FROM installed_mods WHERE instance_id = ?1 ORDER BY name",
    )?;

    let mods = stmt
        .query_map([instance_id], |row| {
            Ok(InstalledMod {
                id: row.get(0)?,
                instance_id: row.get(1)?,
                mod_id: row.get(2)?,
                source: row.get(3)?,
                name: row.get(4)?,
                version: row.get(5)?,
                file_name: row.get(6)?,
                enabled: row.get::<_, i32>(7)? != 0,
                installed_at: row.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(mods)
}

/// Record a mod installation.
pub fn record_mod_install(
    db: &rusqlite::Connection,
    instance_id: &str,
    mod_id: &str,
    source: &str,
    name: &str,
    version: &str,
    file_name: &str,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    db.execute(
        "INSERT INTO installed_mods (instance_id, mod_id, source, name, version, file_name, enabled, installed_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)",
        rusqlite::params![instance_id, mod_id, source, name, version, file_name, now],
    )?;
    Ok(())
}

/// Toggle a mod's enabled state (renames the file to .disabled or back).
pub fn toggle_mod(db: &rusqlite::Connection, mod_id: i64, mods_dir: &std::path::Path) -> Result<bool> {
    let mods = get_mod_by_id(db, mod_id)?;
    let mod_info = mods.ok_or_else(|| anyhow::anyhow!("Mod not found"))?;

    let current_path = mods_dir.join(&mod_info.file_name);
    let new_enabled = !mod_info.enabled;

    let new_filename = if new_enabled {
        // Enable: remove .disabled suffix
        mod_info.file_name.replace(".disabled", "")
    } else {
        // Disable: add .disabled suffix
        format!("{}.disabled", mod_info.file_name)
    };
    let new_path = mods_dir.join(&new_filename);

    // Rename file on disk
    if current_path.exists() {
        std::fs::rename(&current_path, &new_path)?;
    }

    // Update database
    db.execute(
        "UPDATE installed_mods SET enabled = ?2, file_name = ?3 WHERE id = ?1",
        rusqlite::params![mod_id, new_enabled as i32, new_filename],
    )?;

    Ok(new_enabled)
}

/// Remove a mod from an instance (delete file + DB record).
pub fn remove_mod(db: &rusqlite::Connection, mod_id: i64, mods_dir: &std::path::Path) -> Result<()> {
    let mod_info = get_mod_by_id(db, mod_id)?
        .ok_or_else(|| anyhow::anyhow!("Mod not found"))?;

    // Delete file from disk
    let file_path = mods_dir.join(&mod_info.file_name);
    if file_path.exists() {
        std::fs::remove_file(&file_path)?;
    }

    // Remove DB record
    db.execute("DELETE FROM installed_mods WHERE id = ?1", [mod_id])?;
    Ok(())
}

/// Check if a mod is already installed for an instance.
pub fn is_mod_installed(
    db: &rusqlite::Connection,
    instance_id: &str,
    mod_id: &str,
    source: &str,
) -> Result<bool> {
    let mut stmt = db.prepare(
        "SELECT COUNT(*) FROM installed_mods WHERE instance_id = ?1 AND mod_id = ?2 AND source = ?3",
    )?;
    let count: i64 = stmt.query_row(rusqlite::params![instance_id, mod_id, source], |row| {
        row.get(0)
    })?;
    Ok(count > 0)
}

fn get_mod_by_id(db: &rusqlite::Connection, mod_id: i64) -> Result<Option<InstalledMod>> {
    let mut stmt = db.prepare(
        "SELECT id, instance_id, mod_id, source, name, version, file_name, enabled, installed_at 
         FROM installed_mods WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map([mod_id], |row| {
        Ok(InstalledMod {
            id: row.get(0)?,
            instance_id: row.get(1)?,
            mod_id: row.get(2)?,
            source: row.get(3)?,
            name: row.get(4)?,
            version: row.get(5)?,
            file_name: row.get(6)?,
            enabled: row.get::<_, i32>(7)? != 0,
            installed_at: row.get(8)?,
        })
    })?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}
