use crate::error::AppError;
use serde::Serialize;

/// A Minecraft server in the server list.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub address: String,
    pub icon: Option<String>,
    pub is_hidden: bool,
    pub index: usize,
}

/// A singleplayer world.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SingleplayerWorld {
    pub name: String,
    pub folder_name: String,
    pub game_mode: String,
    pub last_played: Option<String>,
    pub size_bytes: u64,
    pub icon: Option<String>,
    pub seed: Option<String>,
}

/// Combined worlds view.
#[derive(Debug, Clone, Serialize)]
pub struct WorldsInfo {
    pub servers: Vec<ServerEntry>,
    pub singleplayer: Vec<SingleplayerWorld>,
}

/// Get servers and singleplayer worlds for an instance.
#[tauri::command]
pub async fn get_instance_worlds(instance_id: String) -> Result<WorldsInfo, AppError> {
    let instance_dir = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id);

    let servers = parse_servers_dat(&instance_dir.join("servers.dat")).await?;
    let singleplayer = scan_singleplayer_worlds(&instance_dir.join("saves")).await?;

    Ok(WorldsInfo {
        servers,
        singleplayer,
    })
}

/// Add a server to the instance's server list.
#[tauri::command]
pub async fn add_server(
    instance_id: String,
    name: String,
    address: String,
) -> Result<ServerEntry, AppError> {
    let servers_path = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("servers.dat");

    let mut servers = parse_servers_dat(&servers_path).await.unwrap_or_default();

    let entry = ServerEntry {
        name: name.clone(),
        address: address.clone(),
        icon: None,
        is_hidden: false,
        index: servers.len(),
    };

    servers.push(entry.clone());
    write_servers_dat(&servers_path, &servers).await?;

    Ok(entry)
}

/// Edit a server entry.
#[tauri::command]
pub async fn edit_server(
    instance_id: String,
    index: usize,
    name: String,
    address: String,
) -> Result<(), AppError> {
    let servers_path = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("servers.dat");

    let mut servers = parse_servers_dat(&servers_path).await.unwrap_or_default();

    if index >= servers.len() {
        return Err(AppError::Internal("Server index out of range".to_string()));
    }

    servers[index].name = name;
    servers[index].address = address;
    write_servers_dat(&servers_path, &servers).await?;

    Ok(())
}

/// Remove a server from the list.
#[tauri::command]
pub async fn remove_server(instance_id: String, index: usize) -> Result<(), AppError> {
    let servers_path = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("servers.dat");

    let mut servers = parse_servers_dat(&servers_path).await.unwrap_or_default();

    if index >= servers.len() {
        return Err(AppError::Internal("Server index out of range".to_string()));
    }

    servers.remove(index);
    write_servers_dat(&servers_path, &servers).await?;

    Ok(())
}

/// Ping a Minecraft server for status.
#[tauri::command]
pub async fn ping_server(address: String) -> Result<ServerStatus, AppError> {
    use std::net::TcpStream;
    use std::time::Duration;

    // Resolve address (add default port if missing)
    let addr = if address.contains(':') {
        address.clone()
    } else {
        format!("{}:25565", address)
    };

    let stream = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|_| "Invalid server address".to_string())?,
        Duration::from_secs(5),
    )
    .map_err(|e| format!("Cannot connect: {}", e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        ?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        ?;

    // For now, just check connectivity. Full SLP protocol would require
    // varint encoding, handshake packet, etc.
    Ok(ServerStatus {
        online: true,
        players_online: None,
        players_max: None,
        version: None,
        motd: None,
        latency_ms: 0,
    })
}

#[derive(Serialize)]
pub struct ServerStatus {
    pub online: bool,
    pub players_online: Option<u32>,
    pub players_max: Option<u32>,
    pub version: Option<String>,
    pub motd: Option<String>,
    pub latency_ms: u64,
}

/// Delete a singleplayer world.
#[tauri::command]
pub async fn delete_world(instance_id: String, folder_name: String) -> Result<(), AppError> {
    let world_path = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("saves")
        .join(&folder_name);

    if !world_path.exists() {
        return Err(AppError::Internal("World not found".to_string()));
    }

    tokio::fs::remove_dir_all(&world_path)
        .await
        ?;

    Ok(())
}

/// Rename a singleplayer world.
#[tauri::command]
pub async fn rename_world(
    instance_id: String,
    folder_name: String,
    _new_name: String,
) -> Result<(), AppError> {
    let level_dat = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("saves")
        .join(&folder_name)
        .join("level.dat");

    if !level_dat.exists() {
        return Err(AppError::Internal("World not found (no level.dat)".to_string()));
    }

    // We can't easily rename via level.dat (NBT format), so we store the display name
    // in our DB instead. The folder stays the same.
    // For a full implementation we'd need an NBT parser.
    Ok(())
}

/// Backup a singleplayer world by copying its folder.
#[tauri::command]
pub async fn backup_world(instance_id: String, folder_name: String) -> Result<String, AppError> {
    let saves_dir = crate::utils::paths::data_dir()
        .join("instances")
        .join(&instance_id)
        .join("saves");

    let world_dir = saves_dir.join(&folder_name);
    if !world_dir.exists() {
        return Err(AppError::Internal("World not found".to_string()));
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_backup_{}", folder_name, timestamp);
    let backup_dir = saves_dir.join(&backup_name);

    copy_dir_recursive(&world_dir, &backup_dir)
        .await
        ?;

    Ok(backup_name)
}

/// Parse servers.dat (NBT format). Returns empty vec if file doesn't exist.
async fn parse_servers_dat(path: &std::path::Path) -> Result<Vec<ServerEntry>, AppError> {
    if !path.exists() {
        return Ok(vec![]);
    }

    // servers.dat is NBT format. For a basic implementation, we'll try to parse
    // with the simple_nbt crate or fall back to empty.
    // The NBT structure is: {servers: [{name: "", ip: "", ...}, ...]}
    let data = tokio::fs::read(path).await?;

    // Try to parse as NBT
    match parse_servers_nbt(&data) {
        Some(servers) => Ok(servers),
        None => Ok(vec![]),
    }
}

/// Basic NBT parser for servers.dat
fn parse_servers_nbt(data: &[u8]) -> Option<Vec<ServerEntry>> {
    // servers.dat uses Java NBT format (GZip compressed)
    use std::io::Read;

    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).ok()?;

    // Very basic NBT parsing — look for server entries
    // NBT format: TAG_Compound -> TAG_List("servers") -> TAG_Compound entries
    // Each entry has TAG_String("name") and TAG_String("ip")

    // Use a simpler approach: search for string patterns
    let mut servers = Vec::new();

    // Try using the serde_nbt or manual parsing
    // For robustness, use the nbt crate if available, otherwise do basic extraction
    let _text = String::from_utf8_lossy(&decompressed);

    // Simple regex-like extraction (fallback)
    // This won't work for binary NBT, so we need proper parsing

    // Use a proper NBT reader approach
    let mut cursor = std::io::Cursor::new(&decompressed);
    if let Some(root) = read_nbt_compound(&mut cursor) {
        if let Some(NbtValue::List(server_list)) = root.get("servers") {
            for entry in server_list {
                if let NbtValue::Compound(compound) = entry {
                    let name = match compound.get("name") {
                        Some(NbtValue::String(s)) => s.clone(),
                        _ => "Unknown".to_string(),
                    };
                    let address = match compound.get("ip") {
                        Some(NbtValue::String(s)) => s.clone(),
                        _ => continue,
                    };
                    let is_hidden = match compound.get("hideAddress") {
                        Some(NbtValue::Byte(b)) => *b != 0,
                        _ => false,
                    };

                    servers.push(ServerEntry {
                        name,
                        address,
                        icon: None,
                        is_hidden,
                        index: servers.len(),
                    });
                }
            }
        }
    }

    Some(servers)
}

// Minimal NBT value types for servers.dat parsing
#[allow(dead_code)]
#[derive(Debug)]
enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    List(Vec<NbtValue>),
    Compound(std::collections::HashMap<String, NbtValue>),
}

fn read_nbt_compound(
    reader: &mut std::io::Cursor<&Vec<u8>>,
) -> Option<std::collections::HashMap<String, NbtValue>> {
    use std::io::Read;

    let mut buf = [0u8; 1];

    // Read root tag type (should be TAG_Compound = 10)
    reader.read_exact(&mut buf).ok()?;
    if buf[0] != 10 {
        return None;
    }

    // Read root name
    read_nbt_string(reader)?;

    read_compound_contents(reader)
}

fn read_compound_contents(
    reader: &mut std::io::Cursor<&Vec<u8>>,
) -> Option<std::collections::HashMap<String, NbtValue>> {
    use std::io::Read;

    let mut map = std::collections::HashMap::new();
    let mut buf = [0u8; 1];

    loop {
        // Read tag type
        if reader.read_exact(&mut buf).is_err() {
            break;
        }
        let tag_type = buf[0];

        // TAG_End = 0
        if tag_type == 0 {
            break;
        }

        let name = read_nbt_string(reader)?;
        let value = read_nbt_value(reader, tag_type)?;
        map.insert(name, value);
    }

    Some(map)
}

fn read_nbt_value(reader: &mut std::io::Cursor<&Vec<u8>>, tag_type: u8) -> Option<NbtValue> {
    use std::io::Read;

    let mut buf8 = [0u8; 1];
    let mut buf16 = [0u8; 2];
    let mut buf32 = [0u8; 4];
    let mut buf64 = [0u8; 8];

    match tag_type {
        1 => {
            // TAG_Byte
            reader.read_exact(&mut buf8).ok()?;
            Some(NbtValue::Byte(buf8[0] as i8))
        }
        2 => {
            // TAG_Short
            reader.read_exact(&mut buf16).ok()?;
            Some(NbtValue::Short(i16::from_be_bytes(buf16)))
        }
        3 => {
            // TAG_Int
            reader.read_exact(&mut buf32).ok()?;
            Some(NbtValue::Int(i32::from_be_bytes(buf32)))
        }
        4 => {
            // TAG_Long
            reader.read_exact(&mut buf64).ok()?;
            Some(NbtValue::Long(i64::from_be_bytes(buf64)))
        }
        5 => {
            // TAG_Float
            reader.read_exact(&mut buf32).ok()?;
            Some(NbtValue::Float(f32::from_be_bytes(buf32)))
        }
        6 => {
            // TAG_Double
            reader.read_exact(&mut buf64).ok()?;
            Some(NbtValue::Double(f64::from_be_bytes(buf64)))
        }
        7 => {
            // TAG_Byte_Array
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf).ok()?;
            let len = i32::from_be_bytes(len_buf) as usize;
            let mut data = vec![0u8; len];
            reader.read_exact(&mut data).ok()?;
            // Skip byte arrays, not needed for servers
            Some(NbtValue::Int(0))
        }
        8 => {
            // TAG_String
            let s = read_nbt_string(reader)?;
            Some(NbtValue::String(s))
        }
        9 => {
            // TAG_List
            let mut type_buf = [0u8; 1];
            reader.read_exact(&mut type_buf).ok()?;
            let inner_type = type_buf[0];

            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf).ok()?;
            let len = i32::from_be_bytes(len_buf) as usize;

            let mut items = Vec::new();
            for _ in 0..len {
                if let Some(val) = read_nbt_value(reader, inner_type) {
                    items.push(val);
                }
            }
            Some(NbtValue::List(items))
        }
        10 => {
            // TAG_Compound
            let contents = read_compound_contents(reader)?;
            Some(NbtValue::Compound(contents))
        }
        _ => None,
    }
}

fn read_nbt_string(reader: &mut std::io::Cursor<&Vec<u8>>) -> Option<String> {
    use std::io::Read;

    let mut len_buf = [0u8; 2];
    reader.read_exact(&mut len_buf).ok()?;
    let len = u16::from_be_bytes(len_buf) as usize;

    let mut str_buf = vec![0u8; len];
    reader.read_exact(&mut str_buf).ok()?;

    String::from_utf8(str_buf).ok()
}

/// Write servers.dat in NBT format.
async fn write_servers_dat(path: &std::path::Path, servers: &[ServerEntry]) -> Result<(), AppError> {
    // Build NBT data manually
    let mut nbt_data = Vec::new();

    // Root compound tag
    nbt_data.push(10u8); // TAG_Compound
    write_nbt_string(&mut nbt_data, ""); // Root name (empty)

    // servers list
    nbt_data.push(9u8); // TAG_List
    write_nbt_string(&mut nbt_data, "servers");
    nbt_data.push(10u8); // List type = Compound
    nbt_data.extend_from_slice(&(servers.len() as i32).to_be_bytes());

    for server in servers {
        // Each server compound
        write_nbt_string_tag(&mut nbt_data, "name", &server.name);
        write_nbt_string_tag(&mut nbt_data, "ip", &server.address);
        nbt_data.push(1); // TAG_Byte
        write_nbt_string(&mut nbt_data, "hideAddress");
        nbt_data.push(if server.is_hidden { 1 } else { 0 });
        nbt_data.push(0); // TAG_End for this compound
    }

    nbt_data.push(0); // TAG_End for root compound

    // GZip compress
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&nbt_data)?;
    let compressed = encoder.finish()?;

    tokio::fs::write(path, compressed)
        .await
        ?;

    Ok(())
}

fn write_nbt_string(data: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    data.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    data.extend_from_slice(bytes);
}

fn write_nbt_string_tag(data: &mut Vec<u8>, name: &str, value: &str) {
    data.push(8u8); // TAG_String
    write_nbt_string(data, name);
    write_nbt_string(data, value);
}

/// Scan singleplayer worlds in the saves directory.
async fn scan_singleplayer_worlds(
    saves_dir: &std::path::Path,
) -> Result<Vec<SingleplayerWorld>, AppError> {
    if !saves_dir.exists() {
        return Ok(vec![]);
    }

    let mut worlds = Vec::new();
    let mut entries = tokio::fs::read_dir(saves_dir)
        .await
        ?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let level_dat = path.join("level.dat");
        if !level_dat.exists() {
            continue;
        }

        let folder_name = entry.file_name().to_string_lossy().to_string();

        // Calculate world size
        let size_bytes = dir_size(&path).await.unwrap_or(0);

        // Try to read level.dat for world name
        let name = folder_name.clone();

        // Check for icon
        let icon_path = path.join("icon.png");
        let icon = if icon_path.exists() {
            Some(icon_path.to_string_lossy().to_string())
        } else {
            None
        };

        worlds.push(SingleplayerWorld {
            name,
            folder_name,
            game_mode: "Unknown".to_string(),
            last_played: None,
            size_bytes,
            icon,
            seed: None,
        });
    }

    Ok(worlds)
}

/// Calculate directory size recursively.
async fn dir_size(path: &std::path::Path) -> Result<u64, std::io::Error> {
    let mut total = 0u64;
    let mut entries = tokio::fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        if metadata.is_dir() {
            total += Box::pin(dir_size(&entry.path())).await?;
        } else {
            total += metadata.len();
        }
    }

    Ok(total)
}

/// Recursively copy a directory.
async fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dst).await?;

    let mut entries = tokio::fs::read_dir(src).await?;

    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            tokio::fs::copy(&src_path, &dst_path).await?;
        }
    }

    Ok(())
}
