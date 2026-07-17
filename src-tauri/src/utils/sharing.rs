// Instance sharing — export/import instances as portable share codes.
//
// Architecture:
// - Export: serialize instance config + installed mods → JSON → gzip → base64
// - Import: base64 decode → gunzip → JSON parse → create instance + download mods
// - Share codes are self-contained (no server needed) — works via Discord, pastebin, etc.
// - Format versioning for forward compatibility

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const FORMAT_VERSION: u32 = 1;

/// The portable share payload — everything needed to recreate an instance.
#[derive(Debug, Serialize, Deserialize)]
pub struct SharePayload {
    /// Format version for forward compatibility.
    pub version: u32,
    /// Instance display name.
    pub name: String,
    /// Minecraft version (e.g., "1.21.4").
    pub game_version: String,
    /// Mod loader type (vanilla, fabric, forge, neoforge, quilt).
    pub loader: String,
    /// Mod loader version.
    pub loader_version: String,
    /// Allocated memory in MB.
    pub allocated_memory_mb: i64,
    /// Custom JVM arguments.
    pub java_args: Option<String>,
    /// Resolution (e.g., "1920x1080").
    pub resolution: Option<String>,
    /// User notes about the instance.
    pub notes: Option<String>,
    /// Installed mods with their source info.
    pub mods: Vec<SharedMod>,
}

/// A mod entry in a shared instance.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SharedMod {
    /// Source platform ("modrinth" or "curseforge").
    pub source: String,
    /// Project ID on the source platform.
    pub project_id: String,
    /// Specific version ID to install.
    pub version_id: String,
    /// Display name.
    pub name: String,
    /// Filename of the installed mod.
    pub file_name: String,
}

/// Export an instance (and its installed mods) to a share code string.
pub fn export_instance(
    name: &str,
    game_version: &str,
    loader: &str,
    loader_version: &str,
    allocated_memory_mb: i64,
    java_args: Option<&str>,
    resolution: Option<&str>,
    notes: Option<&str>,
    mods: Vec<SharedMod>,
) -> Result<String> {
    let payload = SharePayload {
        version: FORMAT_VERSION,
        name: name.to_string(),
        game_version: game_version.to_string(),
        loader: loader.to_string(),
        loader_version: loader_version.to_string(),
        allocated_memory_mb,
        java_args: java_args.map(String::from),
        resolution: resolution.map(String::from),
        notes: notes.map(String::from),
        mods,
    };

    // Serialize to JSON
    let json = serde_json::to_vec(&payload).context("Failed to serialize share payload")?;

    // Compress with gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&json).context("Failed to compress share data")?;
    let compressed = encoder.finish().context("Failed to finish gzip compression")?;

    // Encode as base64
    let encoded = base64_encode(&compressed);

    // Add a prefix for identification
    Ok(format!("OMC:{}", encoded))
}

/// Import an instance from a share code string.
pub fn import_instance(share_code: &str) -> Result<SharePayload> {
    // Strip prefix
    let encoded = share_code
        .strip_prefix("OMC:")
        .unwrap_or(share_code);

    // Decode base64
    let compressed = base64_decode(encoded).context("Invalid base64 in share code")?;

    // Decompress gzip
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut json = Vec::new();
    decoder.read_to_end(&mut json).context("Failed to decompress share data")?;

    // Parse JSON
    let payload: SharePayload =
        serde_json::from_slice(&json).context("Failed to parse share payload")?;

    // Validate format version
    if payload.version > FORMAT_VERSION {
        anyhow::bail!(
            "Share code format v{} is newer than supported v{}. Please update OmniLauncherMC.",
            payload.version,
            FORMAT_VERSION
        );
    }

    // Validate required fields
    if payload.name.is_empty() {
        anyhow::bail!("Share code has empty instance name");
    }
    if payload.game_version.is_empty() {
        anyhow::bail!("Share code has empty game version");
    }

    Ok(payload)
}

// ── Base64 helpers (no external crate needed) ─────────────────

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut result = String::with_capacity((data.len() * 4 / 3) + 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(B64[((triple >> 18) & 0x3F) as usize] as char);
        result.push(B64[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(B64[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(B64[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let input: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let input = input.trim_end_matches('=');

    let mut result = Vec::with_capacity(input.len() * 3 / 4);

    for chunk in input.as_bytes().chunks(4) {
        if chunk.len() < 2 {
            break;
        }

        let b0 = b64_val(chunk[0]).context("Invalid base64 character")?;
        let b1 = b64_val(chunk[1]).context("Invalid base64 character")?;

        let triple = (b0 as u32) << 18
            | (b1 as u32) << 12
            | if chunk.len() > 2 {
                b64_val(chunk[2]).unwrap_or(0) as u32
            } else {
                0
            } << 6
            | if chunk.len() > 3 {
                b64_val(chunk[3]).unwrap_or(0) as u32
            } else {
                0
            };

        result.push((triple >> 16) as u8);
        if chunk.len() > 2 {
            result.push((triple >> 8 & 0xFF) as u8);
        }
        if chunk.len() > 3 {
            result.push((triple & 0xFF) as u8);
        }
    }

    Ok(result)
}

fn b64_val(c: u8) -> Result<u8> {
    match c {
        b'A'..=b'Z' => Ok(c - b'A'),
        b'a'..=b'z' => Ok(c - b'a' + 26),
        b'0'..=b'9' => Ok(c - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => anyhow::bail!("Invalid base64 byte: {}", c as char),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let mods = vec![
            SharedMod {
                source: "modrinth".into(),
                project_id: "AANobbMI".into(),
                version_id: "v1".into(),
                name: "Sodium".into(),
                file_name: "sodium-0.6.0.jar".into(),
            },
            SharedMod {
                source: "modrinth".into(),
                project_id: "YL57xq9U".into(),
                version_id: "v2".into(),
                name: "Iris".into(),
                file_name: "iris-1.7.0.jar".into(),
            },
        ];

        let code = export_instance(
            "My Modpack",
            "1.21.4",
            "fabric",
            "0.16.14",
            8192,
            Some("-XX:+UseG1GC"),
            Some("1920x1080"),
            Some("My cool modpack"),
            mods.clone(),
        )
        .unwrap();

        assert!(code.starts_with("OMC:"));

        let payload = import_instance(&code).unwrap();
        assert_eq!(payload.name, "My Modpack");
        assert_eq!(payload.game_version, "1.21.4");
        assert_eq!(payload.loader, "fabric");
        assert_eq!(payload.mods.len(), 2);
        assert_eq!(payload.mods[0].name, "Sodium");
        assert_eq!(payload.allocated_memory_mb, 8192);
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, OmniLauncherMC!";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_empty_instance() {
        let code = export_instance("Empty", "1.20.1", "vanilla", "", 4096, None, None, None, vec![])
            .unwrap();
        let payload = import_instance(&code).unwrap();
        assert_eq!(payload.name, "Empty");
        assert!(payload.mods.is_empty());
    }

    #[test]
    fn test_no_prefix_import() {
        let data = b"test data";
        let encoded = base64_encode(data);
        // Import without OMC: prefix should still work
        let payload_str = format!(
            "{{\"version\":1,\"name\":\"T\",\"game_version\":\"1.20.1\",\"loader\":\"vanilla\",\"loader_version\":\"\",\"allocated_memory_mb\":4096,\"java_args\":null,\"resolution\":null,\"notes\":null,\"mods\":[]}}"
        );
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(payload_str.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();
        let code = base64_encode(&compressed);
        let result = import_instance(&code);
        assert!(result.is_ok());
    }
}
