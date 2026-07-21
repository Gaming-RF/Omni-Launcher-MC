use anyhow::Result;
use rusqlite::Connection;

/// Run all database migrations. Safe to call multiple times.
pub fn run_migrations(db: &Connection) -> Result<()> {
    // Enable WAL mode for better concurrent read/write performance.
    // WAL allows readers to proceed while a write is in progress.
    db.execute_batch("PRAGMA journal_mode=WAL;")?;

    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS accounts (
            uuid TEXT PRIMARY KEY NOT NULL,
            username TEXT NOT NULL,
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            skin_url TEXT
        );

        CREATE TABLE IF NOT EXISTS instances (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            game_version TEXT NOT NULL,
            loader TEXT NOT NULL DEFAULT 'vanilla',
            loader_version TEXT,
            icon TEXT,
            created_at TEXT NOT NULL,
            last_played TEXT,
            play_time_secs INTEGER NOT NULL DEFAULT 0,
            java_args TEXT,
            resolution TEXT,
            notes TEXT,
            groups TEXT,
            allocated_memory_mb INTEGER NOT NULL DEFAULT 4096
        );

        CREATE TABLE IF NOT EXISTS installed_mods (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            instance_id TEXT NOT NULL,
            mod_id TEXT NOT NULL,
            source TEXT NOT NULL,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            file_name TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            installed_at TEXT NOT NULL,
            FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );

        -- Create default settings if they don't exist
        INSERT OR IGNORE INTO settings (key, value) VALUES ('default_memory_mb', '4096');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'dark');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('language', 'en');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('default_resolution', '1920x1080');

        -- Java installations table (for multi-Java management)
        CREATE TABLE IF NOT EXISTS java_installations (
            id TEXT PRIMARY KEY NOT NULL,
            path TEXT NOT NULL,
            major_version INTEGER NOT NULL,
            arch TEXT NOT NULL,
            vendor TEXT NOT NULL,
            is_auto_downloaded INTEGER NOT NULL DEFAULT 0,
            detected_at TEXT NOT NULL
        );",
    )?;

    // Add per-instance Java override column if missing.
    // SQLite doesn't support IF NOT EXISTS for ALTER TABLE, so we catch the error.
    match db.execute_batch("ALTER TABLE instances ADD COLUMN java_installation_id TEXT;") {
        Ok(()) => log::info!("Added java_installation_id column to instances table"),
        Err(_) => {
            // Column already exists — this is expected on subsequent runs
        }
    }

    // Add last_used column to accounts for multi-account switching.
    match db.execute_batch("ALTER TABLE accounts ADD COLUMN last_used TEXT;") {
        Ok(()) => log::info!("Added last_used column to accounts table"),
        Err(_) => {
            // Column already exists — this is expected on subsequent runs
        }
    }

    Ok(())
}
