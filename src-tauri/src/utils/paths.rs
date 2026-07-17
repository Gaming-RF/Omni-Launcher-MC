use std::path::PathBuf;

/// Get the base data directory for OmniLauncherMC.
pub fn data_dir() -> PathBuf {
    let base = dirs::data_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("OmniLauncherMC")
}

/// Get the path to the SQLite database file.
pub fn db_path() -> PathBuf {
    data_dir().join("omnilaunchermc.db")
}

/// Get the path to the instances directory.
pub fn instances_dir() -> PathBuf {
    data_dir().join("instances")
}

/// Get the path to the Minecraft version files directory.
pub fn versions_dir() -> PathBuf {
    data_dir().join("versions")
}

/// Get the path to the libraries directory.
pub fn libraries_dir() -> PathBuf {
    data_dir().join("libraries")
}

/// Get the path to the assets directory.
pub fn assets_dir() -> PathBuf {
    data_dir().join("assets")
}
