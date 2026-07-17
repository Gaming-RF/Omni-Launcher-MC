// Instance sharing — export/import as portable base64 share codes.
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct SharePayload {
    pub version: u32,
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: String,
    pub allocated_memory_mb: i64,
    pub java_args: Option<String>,
    pub mods: Vec<SharedMod>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SharedMod {
    pub source: String,
    pub project_id: String,
    pub version_id: String,
    pub name: String,
    pub file_name: String,
}

pub fn export_instance(name: &str, game_version: &str, loader: &str, loader_version: &str, allocated_memory_mb: i64, java_args: Option<&str>, mods: Vec<SharedMod>) -> Result<String> {
    let payload = SharePayload { version: FORMAT_VERSION, name: name.to_string(), game_version: game_version.to_string(), loader: loader.to_string(), loader_version: loader_version.to_string(), allocated_memory_mb, java_args: java_args.map(String::from), mods };
    let json = serde_json::to_vec(&payload)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&json)?;
    let compressed = encoder.finish()?;
    Ok(format!("OMC:{}", base64_encode(&compressed)))
}

pub fn import_instance(share_code: &str) -> Result<SharePayload> {
    let encoded = share_code.strip_prefix("OMC:").unwrap_or(share_code);
    let compressed = base64_decode(encoded).context("Invalid base64")?;
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut json = Vec::new();
    decoder.read_to_end(&mut json)?;
    let payload: SharePayload = serde_json::from_slice(&json)?;
    if payload.version > FORMAT_VERSION { anyhow::bail!("Newer format version {}. Update OmniLauncherMC.", payload.version); }
    Ok(payload)
}

fn base64_encode(data: &[u8]) -> String {
    const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut r = String::with_capacity((data.len() * 4 / 3) + 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let t = (b0 << 16) | (b1 << 8) | b2;
        r.push(B64[((t >> 18) & 0x3F) as usize] as char);
        r.push(B64[((t >> 12) & 0x3F) as usize] as char);
        r.push(if chunk.len() > 1 { B64[((t >> 6) & 0x3F) as usize] as char } else { '=' });
        r.push(if chunk.len() > 2 { B64[(t & 0x3F) as usize] as char } else { '=' });
    }
    r
}

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let input: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let input = input.trim_end_matches('=');
    let mut r = Vec::with_capacity(input.len() * 3 / 4);
    for chunk in input.as_bytes().chunks(4) {
        if chunk.len() < 2 { break; }
        let b0 = b64v(chunk[0])?; let b1 = b64v(chunk[1])?;
        let t = (b0 as u32) << 18 | (b1 as u32) << 12
            | if chunk.len() > 2 { b64v(chunk[2]).unwrap_or(0) as u32 } else { 0 } << 6
            | if chunk.len() > 3 { b64v(chunk[3]).unwrap_or(0) as u32 } else { 0 };
        r.push((t >> 16) as u8);
        if chunk.len() > 2 { r.push((t >> 8 & 0xFF) as u8); }
        if chunk.len() > 3 { r.push((t & 0xFF) as u8); }
    }
    Ok(r)
}
fn b64v(c: u8) -> Result<u8> {
    match c { b'A'..=b'Z' => Ok(c - b'A'), b'a'..=b'z' => Ok(c - b'a' + 26), b'0'..=b'9' => Ok(c - b'0' + 52), b'+' => Ok(62), b'/' => Ok(63), _ => anyhow::bail!("Invalid b64") }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_roundtrip() {
        let mods = vec![SharedMod { source: "modrinth".into(), project_id: "abc".into(), version_id: "v1".into(), name: "Sodium".into(), file_name: "sodium.jar".into() }];
        let code = export_instance("Test", "1.21.4", "fabric", "0.16", 8192, Some("-XX:+UseG1GC"), mods).unwrap();
        assert!(code.starts_with("OMC:"));
        let p = import_instance(&code).unwrap();
        assert_eq!(p.name, "Test");
        assert_eq!(p.mods.len(), 1);
    }
}
