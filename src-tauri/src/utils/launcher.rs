use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::api::minecraft;
use crate::db::instances::GameInstance;
use std::process::Stdio;

/// The main game launcher. Handles downloading version files, assets, libraries,
/// assembling classpath, building JVM/game args, and launching the process.
pub struct GameLauncher {
    pub base_dir: PathBuf,
    pub java_path: PathBuf,
}

impl GameLauncher {
    pub fn new(base_dir: PathBuf, java_path: PathBuf) -> Self {
        Self { base_dir, java_path }
    }

    fn versions_dir(&self) -> PathBuf {
        self.base_dir.join("versions")
    }

    fn libraries_dir(&self) -> PathBuf {
        self.base_dir.join("libraries")
    }

    fn assets_dir(&self) -> PathBuf {
        self.base_dir.join("assets")
    }

    fn instance_dir(&self, id: &str) -> PathBuf {
        self.base_dir.join("instances").join(id)
    }

    /// Prepare a game instance for launch: download version JSON, JAR, libraries, assets.
    pub async fn prepare(&self, instance: &GameInstance) -> Result<()> {
        // 1. Fetch version manifest and find the target version
        let manifest = minecraft::fetch_version_manifest().await?;
        let version_entry = manifest
            .versions
            .iter()
            .find(|v| v.id == instance.game_version)
            .with_context(|| format!("Version {} not found in manifest", instance.game_version))?;

        // 2. Download and parse version JSON
        let version_dir = self.versions_dir().join(&instance.game_version);
        let version_json_path = version_dir.join(format!("{}.json", instance.game_version));
        minecraft::download_file(&version_entry.url, &version_json_path).await?;
        let version_json = std::fs::read_to_string(&version_json_path)?;
        let version: minecraft::VersionDetails = serde_json::from_str(&version_json)?;

        // 3. Download client JAR
        let jar_path = version_dir.join(format!("{}.jar", instance.game_version));
        minecraft::download_file_verified(
            &version.downloads.client.url,
            &jar_path,
            &version.downloads.client.sha1,
        )
        .await?;

        // 4. Download libraries
        for lib in &version.libraries {
            if !minecraft::rules_allow(&lib.rules) {
                continue;
            }

            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let path = self.libraries_dir().join(&artifact.path);
                    minecraft::download_file_verified(&artifact.url, &path, &artifact.sha1)
                        .await?;
                }
            }
        }

        // 5. Download assets
        let asset_index_path = self
            .assets_dir()
            .join("indexes")
            .join(format!("{}.json", version.assets));
        minecraft::download_file(&version.asset_index.url, &asset_index_path).await?;
        let index_json = std::fs::read_to_string(&asset_index_path)?;
        let asset_index: minecraft::AssetIndexData = serde_json::from_str(&index_json)?;

        for (name, obj) in &asset_index.objects {
            let hash_prefix = &obj.hash[..2];
            let asset_path = self
                .assets_dir()
                .join("objects")
                .join(hash_prefix)
                .join(&obj.hash);
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                hash_prefix, obj.hash
            );
            minecraft::download_file_verified(&url, &asset_path, &obj.hash).await?;
        }

        // 6. Create instance game directory
        let instance_dir = self.instance_dir(&instance.id);
        std::fs::create_dir_all(&instance_dir)?;

        Ok(())
    }

    /// Launch a prepared game instance. Returns the child process PID and the Child handle.
    pub async fn launch(
        &self,
        instance: &GameInstance,
        access_token: &str,
        username: &str,
        uuid: &str,
    ) -> Result<(u32, tokio::process::Child)> {
        let version_dir = self.versions_dir().join(&instance.game_version);
        let version_json_path = version_dir.join(format!("{}.json", instance.game_version));
        let version_json = std::fs::read_to_string(&version_json_path)?;
        let version: minecraft::VersionDetails = serde_json::from_str(&version_json)?;

        // Build classpath
        let mut classpath_entries: Vec<String> = Vec::new();
        for lib in &version.libraries {
            if !minecraft::rules_allow(&lib.rules) {
                continue;
            }
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let path = self.libraries_dir().join(&artifact.path);
                    classpath_entries.push(path.to_string_lossy().to_string());
                }
            }
        }
        // Add client JAR
        let client_jar = version_dir.join(format!("{}.jar", instance.game_version));
        classpath_entries.push(client_jar.to_string_lossy().to_string());
        let classpath = classpath_entries.join(if cfg!(windows) { ";" } else { ":" });

        // Asset index ID
        let assets_id = &version.assets;
        let instance_dir = self.instance_dir(&instance.id);

        // Build JVM arguments
        let mut jvm_args: Vec<String> = vec![
            format!("-Xmx{}M", instance.allocated_memory_mb),
            format!("-Xms{}M", instance.allocated_memory_mb / 2),
            "-Djava.library.path=natives".to_string(),
            format!("-Dminecraft.client.jar={}", client_jar.display()),
        ];

        // Add custom JVM args if set
        if let Some(custom) = &instance.java_args {
            jvm_args.extend(custom.split_whitespace().map(|s| s.to_string()));
        }

        // Build game arguments
        let mut game_args: Vec<String> = Vec::new();

        if let Some(args) = &version.arguments {
            // 1.13+ format
            for arg in &args.game {
                if let Some(s) = arg.as_str() {
                    game_args.push(s.to_string());
                }
            }
        } else if let Some(legacy) = &version.minecraft_arguments {
            // Pre-1.13 format
            game_args = legacy.split_whitespace().map(|s| s.to_string()).collect();
        }

        // Prepare argument replacements
        let replacements: Vec<(&str, &str)> = vec![
            ("${auth_player_name}", username),
            ("${auth_session}", access_token),
            ("${auth_access_token}", access_token),
            ("${auth_uuid}", uuid),
            ("${version_name}", &instance.game_version),
            ("${game_directory}", &instance_dir.to_string_lossy()),
            ("${game_assets}", &self.assets_dir().join("virtual").join("legacy").to_string_lossy()),
            ("${assets_root}", &self.assets_dir().to_string_lossy()),
            ("${assets_index_name}", assets_id),
            ("${user_type}", "msa"),
            ("${user_properties}", "{}"),
            ("${version_type}", &version.version_type),
            ("${launcher_name}", "OmniLauncherMC"),
            ("${launcher_version}", "0.1.0"),
        ];

        // Apply replacements to game args
        for arg in &mut game_args {
            for (placeholder, value) in &replacements {
                if arg.contains(placeholder) {
                    *arg = arg.replace(placeholder, value);
                }
            }
        }

        // Apply JVM arg replacements too
        for arg in &mut jvm_args {
            for (placeholder, value) in &replacements {
                if arg.contains(placeholder) {
                    *arg = arg.replace(placeholder, value);
                }
            }
            // Also handle classpath
            if arg.contains("${classpath}") || arg.contains("${classpath_separator}") {
                *arg = arg.replace("${classpath}", &classpath);
            }
        }

        // If no classpath arg in JVM args, add it
        if !jvm_args.iter().any(|a| a.contains("-cp") || a.contains("classpath")) {
            jvm_args.push("-cp".to_string());
            jvm_args.push(classpath.clone());
        }

        // Construct full command
        let mut cmd_args = jvm_args;
        cmd_args.push(version.main_class.clone());
        cmd_args.extend(game_args);

        log::info!(
            "Launching MC {} with {} args",
            instance.game_version,
            cmd_args.len()
        );

        let child = Command::new(&self.java_path)
            .args(&cmd_args)
            .current_dir(&instance_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start Java process. Is Java installed?")?;

        let pid = child.id().context("Failed to get process ID")?;

        Ok((pid, child))
    }
}

/// Find the Java executable. Checks settings, JAVA_HOME, and common paths.
pub fn find_java(custom_path: Option<&str>) -> Result<PathBuf> {
    // 1. Custom path from settings
    if let Some(path) = custom_path {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // 2. JAVA_HOME environment variable
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let p = PathBuf::from(&java_home).join("bin").join(java_bin_name());
        if p.exists() {
            return Ok(p);
        }
    }

    // 3. Common installation paths
    let candidates = if cfg!(target_os = "windows") {
        vec![
            r"C:\Program Files\Java\jre-21\bin\java.exe",
            r"C:\Program Files\Java\jdk-21\bin\java.exe",
            r"C:\Program Files\Eclipse Adoptium\jdk-21\bin\java.exe",
            r"C:\Program Files\Microsoft\jdk-21\bin\java.exe",
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/Library/Java/JavaVirtualMachines/jdk-21.jdk/Contents/Home/bin/java",
            "/opt/homebrew/opt/openjdk@21/bin/java",
            "/usr/local/opt/openjdk@21/bin/java",
        ]
    } else {
        vec![
            "/usr/lib/jvm/java-21-openjdk/bin/java",
            "/usr/lib/jvm/java-21-openjdk-amd64/bin/java",
            "/usr/bin/java",
        ]
    };

    for path in &candidates {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // 4. Try to find java on PATH
    if let Ok(output) = std::process::Command::new("java").arg("-version").output() {
        if output.status.success() {
            return Ok(PathBuf::from("java"));
        }
    }

    anyhow::bail!(
        "Java not found. Please install Java 21+ or set the Java path in Settings."
    )
}

fn java_bin_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    }
}
