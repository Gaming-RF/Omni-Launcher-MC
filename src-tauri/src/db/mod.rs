use rusqlite::{Connection, Result};

/// Initialize the SQLite database with required tables.
pub fn init(db_path: &std::path::Path) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS accounts (
            uuid TEXT PRIMARY KEY,
            username TEXT NOT NULL,
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            skin_url TEXT,
            created_at INTEGER NOT NULL DEFAULT (unixepoch())
        );

        CREATE TABLE IF NOT EXISTS instances (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            game_version TEXT NOT NULL,
            mod_loader TEXT,
            mod_loader_version TEXT,
            icon TEXT,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            last_played INTEGER,
            play_time_seconds INTEGER NOT NULL DEFAULT 0,
            source TEXT,
            source_id TEXT
        );

        CREATE TABLE IF NOT EXISTS installed_mods (
            id TEXT PRIMARY KEY,
            instance_id TEXT NOT NULL,
            mod_id TEXT NOT NULL,
            source TEXT NOT NULL,
            source_project_id TEXT NOT NULL,
            source_version_id TEXT NOT NULL,
            file_name TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            installed_at INTEGER NOT NULL DEFAULT (unixepoch()),
            FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        ",
    )?;

    log::info!("Database schema initialized");
    Ok(())
}
