use crate::commands::instances::InstanceListItem;
use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Clone)]
pub struct GroupInfo {
    pub name: String,
    pub color: String,
    pub instance_count: i64,
    pub created_at: String,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Parse the comma-separated groups column into a Vec of group names.
/// Returns an empty Vec for NULL or empty strings.
fn parse_groups(raw: &Option<String>) -> Vec<String> {
    raw.as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Join a Vec of group names back into a comma-separated string.
fn join_groups(groups: &[String]) -> Option<String> {
    let joined = groups.join(",");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

/// Create the instance_groups table if it doesn't exist.
/// Called lazily from commands so we don't need to touch migrations.rs.
fn ensure_groups_table(db: &rusqlite::Connection) -> Result<(), String> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS instance_groups (
            name TEXT PRIMARY KEY NOT NULL,
            color TEXT NOT NULL DEFAULT '#6366f1',
            created_at TEXT NOT NULL
        );",
    )
    .map_err(|e| e.to_string())
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_groups(state: State<'_, AppState>) -> Result<Vec<GroupInfo>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_groups_table(&db)?;

    let mut stmt = db
        .prepare(
            "SELECT g.name, g.color, g.created_at,
                    COUNT(CASE WHEN i.groups IS NOT NULL AND i.groups != ''
                                AND (',' || i.groups || ',') LIKE ('%,' || g.name || ',%')
                           THEN 1 END) AS instance_count
             FROM instance_groups g
             LEFT JOIN instances i ON 1=1
             GROUP BY g.name
             ORDER BY g.name",
        )
        .map_err(|e| e.to_string())?;

    let groups = stmt
        .query_map([], |row| {
            Ok(GroupInfo {
                name: row.get(0)?,
                color: row.get(1)?,
                created_at: row.get(2)?,
                instance_count: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(groups)
}

#[tauri::command]
pub fn create_group(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
) -> Result<GroupInfo, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Group name cannot be empty".to_string());
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_groups_table(&db)?;

    let color = color.unwrap_or_else(|| "#6366f1".to_string());
    let created_at = chrono::Utc::now().to_rfc3339();

    db.execute(
        "INSERT INTO instance_groups (name, color, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, color, created_at],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            format!("Group '{}' already exists", name)
        } else {
            e.to_string()
        }
    })?;

    Ok(GroupInfo {
        name,
        color,
        instance_count: 0,
        created_at,
    })
}

#[tauri::command]
pub fn delete_group(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_groups_table(&db)?;

    // Delete the group itself
    db.execute(
        "DELETE FROM instance_groups WHERE name = ?1",
        rusqlite::params![name],
    )
    .map_err(|e| e.to_string())?;

    // Clear this group from all instances
    let mut stmt = db
        .prepare("SELECT id, groups FROM instances WHERE groups IS NOT NULL AND groups != ''")
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, Option<String>)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (id, groups_raw) in rows {
        let mut groups = parse_groups(&groups_raw);
        if let Some(pos) = groups.iter().position(|g| g == &name) {
            groups.remove(pos);
            let new_groups = join_groups(&groups);
            db.execute(
                "UPDATE instances SET groups = ?1 WHERE id = ?2",
                rusqlite::params![new_groups, id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn rename_group(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let new_name = new_name.trim().to_string();
    if new_name.is_empty() {
        return Err("New group name cannot be empty".to_string());
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_groups_table(&db)?;

    // Update the group row
    let changed = db
        .execute(
            "UPDATE instance_groups SET name = ?1 WHERE name = ?2",
            rusqlite::params![new_name, old_name],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                format!("Group '{}' already exists", new_name)
            } else {
                e.to_string()
            }
        })?;

    if changed == 0 {
        return Err(format!("Group '{}' not found", old_name));
    }

    // Update all instances that reference this group
    let mut stmt = db
        .prepare("SELECT id, groups FROM instances WHERE groups IS NOT NULL AND groups != ''")
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, Option<String>)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (id, groups_raw) in rows {
        let mut groups = parse_groups(&groups_raw);
        if let Some(pos) = groups.iter().position(|g| g == &old_name) {
            groups[pos] = new_name.clone();
            let new_groups = join_groups(&groups);
            db.execute(
                "UPDATE instances SET groups = ?1 WHERE id = ?2",
                rusqlite::params![new_groups, id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn update_group_color(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_groups_table(&db)?;

    let changed = db
        .execute(
            "UPDATE instance_groups SET color = ?1 WHERE name = ?2",
            rusqlite::params![color, name],
        )
        .map_err(|e| e.to_string())?;

    if changed == 0 {
        return Err(format!("Group '{}' not found", name));
    }

    Ok(())
}

#[tauri::command]
pub fn assign_instance_to_group(
    state: State<'_, AppState>,
    instance_id: String,
    group_name: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let current: Option<String> = db
        .query_row(
            "SELECT groups FROM instances WHERE id = ?1",
            rusqlite::params![instance_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut groups = parse_groups(&current);
    if !groups.contains(&group_name) {
        groups.push(group_name);
    }
    let new_groups = join_groups(&groups);

    db.execute(
        "UPDATE instances SET groups = ?1 WHERE id = ?2",
        rusqlite::params![new_groups, instance_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_instance_from_group(
    state: State<'_, AppState>,
    instance_id: String,
    group_name: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let current: Option<String> = db
        .query_row(
            "SELECT groups FROM instances WHERE id = ?1",
            rusqlite::params![instance_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut groups = parse_groups(&current);
    groups.retain(|g| g != &group_name);
    let new_groups = join_groups(&groups);

    db.execute(
        "UPDATE instances SET groups = ?1 WHERE id = ?2",
        rusqlite::params![new_groups, instance_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_group_instances(
    state: State<'_, AppState>,
    group_name: String,
) -> Result<Vec<InstanceListItem>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare(
            "SELECT id, name, game_version, loader, loader_version, icon, created_at,
                    last_played, play_time_secs, allocated_memory_mb
             FROM instances
             WHERE groups IS NOT NULL AND groups != ''
               AND (',' || groups || ',') LIKE ('%,' || ?1 || ',%')
             ORDER BY last_played DESC NULLS LAST, created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let instances = stmt
        .query_map(rusqlite::params![group_name], |row| {
            Ok(InstanceListItem {
                id: row.get(0)?,
                name: row.get(1)?,
                game_version: row.get(2)?,
                loader: row.get(3)?,
                loader_version: row.get(4)?,
                icon: row.get(5)?,
                created_at: row.get(6)?,
                last_played: row.get(7)?,
                play_time_secs: row.get(8)?,
                allocated_memory_mb: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(instances)
}
