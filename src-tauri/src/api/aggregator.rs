// Aggregated search — concurrent Modrinth + CurseForge with fuzzy deduplication.
use anyhow::Result;
use serde::{Deserialize, Serialize};

const USER_AGENT: &str = "OmniLauncherMC/0.1.0 (github.com/OmniLauncherMC)";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    pub source: String,
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub icon_url: String,
    pub downloads: u64,
    pub categories: Vec<String>,
}

/// Search both Modrinth and CurseForge concurrently, merge and deduplicate.
pub async fn aggregated_search(
    query: &str,
    modrinth_offset: u32,
    modrinth_limit: u32,
    curseforge_api_key: Option<&str>,
    curseforge_offset: i32,
    curseforge_limit: i32,
) -> Result<Vec<AggregatedResult>> {
    let modrinth_fut = search_modrinth(query, modrinth_offset, modrinth_limit);
    let curseforge_fut = async {
        if let Some(key) = curseforge_api_key {
            search_curseforge(key, query, curseforge_offset, curseforge_limit).await
        } else { Ok(Vec::new()) }
    };

    let (mr, cf) = tokio::join!(modrinth_fut, curseforge_fut);
    let mut all = Vec::new();
    match mr { Ok(r) => all.extend(r), Err(e) => log::warn!("Modrinth search failed: {}", e) }
    match cf { Ok(r) => all.extend(r), Err(e) => log::warn!("CurseForge search failed: {}", e) }
    Ok(deduplicate(all))
}

async fn search_modrinth(query: &str, offset: u32, limit: u32) -> Result<Vec<AggregatedResult>> {
    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let url = format!("https://api.modrinth.com/v2/search?query={}&offset={}&limit={}", urlencoding::encode(query), offset, limit);
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    let hits = resp["hits"].as_array().cloned().unwrap_or_default();
    Ok(hits.into_iter().map(|h| AggregatedResult {
        source: "modrinth".into(),
        project_id: h["project_id"].as_str().unwrap_or_default().into(),
        slug: h["slug"].as_str().unwrap_or_default().into(),
        title: h["title"].as_str().unwrap_or_default().into(),
        description: h["description"].as_str().unwrap_or_default().into(),
        icon_url: h["icon_url"].as_str().unwrap_or_default().into(),
        downloads: h["downloads"].as_u64().unwrap_or(0),
        categories: h["display_categories"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
    }).collect())
}

async fn search_curseforge(api_key: &str, query: &str, offset: i32, limit: i32) -> Result<Vec<AggregatedResult>> {
    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let url = format!("https://api.curseforge.com/v1/mods/search?gameId=432&searchFilter={}&index={}&pageSize={}", urlencoding::encode(query), offset, limit);
    let resp: serde_json::Value = client.get(&url).header("x-api-key", api_key).header("Accept", "application/json").send().await?.json().await?;
    let data = resp["data"].as_array().cloned().unwrap_or_default();
    Ok(data.into_iter().map(|m| AggregatedResult {
        source: "curseforge".into(),
        project_id: m["id"].as_i64().unwrap_or(0).to_string(),
        slug: m["slug"].as_str().unwrap_or_default().into(),
        title: m["name"].as_str().unwrap_or_default().into(),
        description: m["summary"].as_str().unwrap_or_default().into(),
        icon_url: m["logo"]["url"].as_str().unwrap_or_default().into(),
        downloads: m["downloadCount"].as_u64().unwrap_or(0),
        categories: m["categories"].as_array().map(|a| a.iter().filter_map(|c| c["name"].as_str().map(String::from)).collect()).unwrap_or_default(),
    }).collect())
}

fn deduplicate(mut results: Vec<AggregatedResult>) -> Vec<AggregatedResult> {
    results.sort_by(|a, b| b.downloads.cmp(&a.downloads));
    let mut kept: Vec<AggregatedResult> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for result in results {
        let norm = result.title.to_lowercase().chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect::<String>();
        let is_dup = seen.iter().any(|s| jaro_winkler(s, &norm) > 0.88);
        if !is_dup { seen.push(norm); kept.push(result); }
    }
    kept
}

fn jaro_winkler(s1: &str, s2: &str) -> f64 {
    if s1 == s2 { return 1.0; }
    if s1.is_empty() || s2.is_empty() { return 0.0; }
    let (len1, len2) = (s1.len(), s2.len());
    let md = (len1.max(len2) / 2).saturating_sub(1);
    let (b1, b2) = (s1.as_bytes(), s2.as_bytes());
    let (mut sm, mut s2m) = (vec![false; len1], vec![false; len2]);
    let (mut matches, mut trans) = (0usize, 0usize);
    for i in 0..len1 {
        let (start, end) = (if i >= md { i - md } else { 0}, (i + md + 1).min(len2));
        for j in start..end {
            if !s2m[j] && b1[i] == b2[j] { sm[i] = true; s2m[j] = true; matches += 1; break; }
        }
    }
    if matches == 0 { return 0.0; }
    let mut k = 0;
    for i in 0..len1 {
        if !sm[i] { continue; }
        while !s2m[k] { k += 1; }
        if b1[i] != b2[k] { trans += 1; }
        k += 1;
    }
    let m = matches as f64;
    let jaro = (m / len1 as f64 + m / len2 as f64 + (m - trans as f64 / 2.0) / m) / 3.0;
    let prefix = b1.iter().zip(b2.iter()).take(4).take_while(|(a, b)| a == b).count();
    jaro + (prefix as f64 * 0.1 * (1.0 - jaro))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_jaro_winkler_identical() { assert!((jaro_winkler("hello", "hello") - 1.0).abs() < 0.001); }
    #[test]
    fn test_jaro_winkler_similar() { assert!(jaro_winkler("sodium", "sodium fabric") > 0.8); }
    #[test]
    fn test_jaro_winkler_different() { assert!(jaro_winkler("sodium", "iris") < 0.5); }
}
