use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameInstance {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub icon: Option<String>,
    pub created_at: String,
    pub last_played: Option<String>,
    pub play_time_secs: i64,
    pub java_args: Option<String>,
    pub resolution: Option<String>,
    pub notes: Option<String>,
    pub groups: Option<String>,
    pub allocated_memory_mb: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInstanceParams {
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub icon: Option<String>,
    pub java_args: Option<String>,
    pub allocated_memory_mb: i64,
}

/// Get all game instances from the database.
pub fn get_all_instances(db: &rusqlite::Connection) -> Result<Vec<GameInstance>> {
    let mut stmt = db.prepare(
        "SELECT id, name, game_version, loader, loader_version, icon, created_at, 
         last_played, play_time_secs, java_args, resolution, notes, groups, 
         allocated_memory_mb FROM instances ORDER BY last_played DESC, created_at DESC",
    )?;

    let instances = stmt
        .query_map([], |row| {
            Ok(GameInstance {
                id: row.get(0)?,
                name: row.get(1)?,
                game_version: row.get(2)?,
                loader: row.get(3)?,
                loader_version: row.get(4)?,
                icon: row.get(5)?,
                created_at: row.get(6)?,
                last_played: row.get(7)?,
                play_time_secs: row.get(8)?,
                java_args: row.get(9)?,
                resolution: row.get(10)?,
                notes: row.get(11)?,
                groups: row.get(12)?,
                allocated_memory_mb: row.get(13)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(instances)
}

/// Get a single instance by ID.
pub fn get_instance(db: &rusqlite::Connection, id: &str) -> Result<Option<GameInstance>> {
    let mut stmt = db.prepare(
        "SELECT id, name, game_version, loader, loader_version, icon, created_at, 
         last_played, play_time_secs, java_args, resolution, notes, groups, 
         allocated_memory_mb FROM instances WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map([id], |row| {
        Ok(GameInstance {
            id: row.get(0)?,
            name: row.get(1)?,
            game_version: row.get(2)?,
            loader: row.get(3)?,
            loader_version: row.get(4)?,
            icon: row.get(5)?,
            created_at: row.get(6)?,
            last_played: row.get(7)?,
            play_time_secs: row.get(8)?,
            java_args: row.get(9)?,
            resolution: row.get(10)?,
            notes: row.get(11)?,
            groups: row.get(12)?,
            allocated_memory_mb: row.get(13)?,
        })
    })?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Create a new game instance.
pub fn create_instance(
    db: &rusqlite::Connection,
    params: CreateInstanceParams,
) -> Result<GameInstance> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    db.execute(
        "INSERT INTO instances (id, name, game_version, loader, loader_version, icon, 
         created_at, last_played, play_time_secs, java_args, resolution, notes, groups, 
         allocated_memory_mb) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, 0, ?8, NULL, NULL, NULL, ?9)",
        rusqlite::params![
            id,
            params.name,
            params.game_version,
            params.loader,
            params.loader_version,
            params.icon,
            now,
            params.java_args,
            params.allocated_memory_mb,
        ],
    )?;

    Ok(GameInstance {
        id,
        name: params.name,
        game_version: params.game_version,
        loader: params.loader,
        loader_version: params.loader_version,
        icon: params.icon,
        created_at: now,
        last_played: None,
        play_time_secs: 0,
        java_args: params.java_args,
        resolution: None,
        notes: None,
        groups: None,
        allocated_memory_mb: params.allocated_memory_mb,
    })
}

/// Update an existing instance.
pub fn update_instance(
    db: &rusqlite::Connection,
    instance: &GameInstance,
) -> Result<()> {
    db.execute(
        "UPDATE instances SET name = ?2, java_args = ?3, resolution = ?4, notes = ?5, 
         groups = ?6, allocated_memory_mb = ?7, icon = ?8 
         WHERE id = ?1",
        rusqlite::params![
            instance.id,
            instance.name,
            instance.java_args,
            instance.resolution,
            instance.notes,
            instance.groups,
            instance.allocated_memory_mb,
            instance.icon,
        ],
    )?;
    Ok(())
}

/// Delete an instance (does NOT remove files from disk).
pub fn delete_instance(db: &rusqlite::Connection, id: &str) -> Result<()> {
    db.execute("DELETE FROM instances WHERE id = ?1", [id])?;
    Ok(())
}

/// Record that an instance was played.
pub fn record_play(db: &rusqlite::Connection, id: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    db.execute(
        "UPDATE instances SET last_played = ?2 WHERE id = ?1",
        rusqlite::params![id, now],
    )?;
    Ok(())
}

/// Add play time to an instance (in seconds).
pub fn add_play_time(db: &rusqlite::Connection, id: &str, seconds: i64) -> Result<()> {
    db.execute(
        "UPDATE instances SET play_time_secs = play_time_secs + ?2 WHERE id = ?1",
        rusqlite::params![id, seconds],
    )?;
    Ok(())
}
