// Aggregated search — concurrent Modrinth + CurseForge with fuzzy deduplication.
//
// Architecture:
// - Fires both API searches concurrently via tokio::join!
// - Normalizes results into a common AggregatedResult struct
// - Deduplicates by title similarity using Levenshtein distance
// - Sorts by relevance (downloads + title match)
// - Returns unified results with source discriminator

use anyhow::Result;
use serde::{Deserialize, Serialize};

const USER_AGENT: &str = "OmniLauncherMC/0.1.0 (github.com/OmniLauncherMC)";

/// Unified search result from any source.
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
    pub client_side: Option<String>,
    pub server_side: Option<String>,
}

/// Search both Modrinth and CurseForge concurrently, merge and deduplicate results.
pub async fn aggregated_search(
    query: &str,
    modrinth_offset: u32,
    modrinth_limit: u32,
    curseforge_api_key: Option<&str>,
    curseforge_offset: i32,
    curseforge_limit: i32,
) -> Result<Vec<AggregatedResult>> {
    let modrinth_future = search_modrinth(query, modrinth_offset, modrinth_limit);
    let curseforge_future = async {
        if let Some(key) = curseforge_api_key {
            search_curseforge(key, query, curseforge_offset, curseforge_limit).await
        } else {
            Ok(Vec::new())
        }
    };

    let (modrinth_results, curseforge_results) = tokio::join!(modrinth_future, curseforge_future);

    let mut all = Vec::new();

    match modrinth_results {
        Ok(results) => all.extend(results),
        Err(e) => log::warn!("Modrinth search failed: {}", e),
    }

    match curseforge_results {
        Ok(results) => all.extend(results),
        Err(e) => log::warn!("CurseForge search failed: {}", e),
    }

    // Deduplicate by title similarity
    let deduplicated = deduplicate_results(all, query);

    Ok(deduplicated)
}

/// Search Modrinth and normalize results.
async fn search_modrinth(query: &str, offset: u32, limit: u32) -> Result<Vec<AggregatedResult>> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()?;

    let url = format!(
        "https://api.modrinth.com/v2/search?query={}&offset={}&limit={}",
        urlencoding::encode(query),
        offset,
        limit
    );

    let resp: serde_json::Value = client
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    let hits = resp["hits"].as_array().cloned().unwrap_or_default();

    Ok(hits
        .into_iter()
        .map(|hit| AggregatedResult {
            source: "modrinth".to_string(),
            project_id: hit["project_id"].as_str().unwrap_or_default().to_string(),
            slug: hit["slug"].as_str().unwrap_or_default().to_string(),
            title: hit["title"].as_str().unwrap_or_default().to_string(),
            description: hit["description"].as_str().unwrap_or_default().to_string(),
            icon_url: hit["icon_url"].as_str().unwrap_or_default().to_string(),
            downloads: hit["downloads"].as_u64().unwrap_or(0),
            categories: hit["display_categories"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            client_side: hit["client_side"].as_str().map(String::from),
            server_side: hit["server_side"].as_str().map(String::from),
        })
        .collect())
}

/// Search CurseForge and normalize results.
async fn search_curseforge(
    api_key: &str,
    query: &str,
    offset: i32,
    limit: i32,
) -> Result<Vec<AggregatedResult>> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()?;

    let url = format!(
        "https://api.curseforge.com/v1/mods/search?gameId=432&searchFilter={}&index={}&pageSize={}",
        urlencoding::encode(query),
        offset,
        limit
    );

    let resp: serde_json::Value = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("Accept", "application/json")
        .send()
        .await?
        .json()
        .await?;

    let data = resp["data"].as_array().cloned().unwrap_or_default();

    Ok(data
        .into_iter()
        .map(|m| AggregatedResult {
            source: "curseforge".to_string(),
            project_id: m["id"].as_i64().unwrap_or(0).to_string(),
            slug: m["slug"].as_str().unwrap_or_default().to_string(),
            title: m["name"].as_str().unwrap_or_default().to_string(),
            description: m["summary"].as_str().unwrap_or_default().to_string(),
            icon_url: m["logo"]["url"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            downloads: m["downloadCount"].as_u64().unwrap_or(0),
            categories: m["categories"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| c["name"].as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            client_side: None,
            server_side: None,
        })
        .collect())
}

/// Remove duplicate results based on title similarity.
/// Keeps the entry with higher downloads when titles are very similar.
fn deduplicate_results(
    mut results: Vec<AggregatedResult>,
    query: &str,
) -> Vec<AggregatedResult> {
    if results.is_empty() {
        return results;
    }

    // Sort by downloads descending so we keep the most popular when deduping
    results.sort_by(|a, b| b.downloads.cmp(&a.downloads));

    let mut kept: Vec<AggregatedResult> = Vec::new();
    let mut seen_titles: Vec<String> = Vec::new();

    for result in results {
        let normalized = normalize_title(&result.title);

        // Check if we already have a result with a very similar title
        let is_dup = seen_titles.iter().any(|existing| {
            let similarity = jaro_winkler(existing, &normalized);
            similarity > 0.88 // 88% similarity threshold
        });

        if !is_dup {
            seen_titles.push(normalized);
            kept.push(result);
        }
    }

    // Re-sort by relevance: exact query match first, then by downloads
    let query_lower = query.to_lowercase();
    kept.sort_by(|a, b| {
        let a_exact = a.title.to_lowercase().contains(&query_lower);
        let b_exact = b.title.to_lowercase().contains(&query_lower);
        b_exact
            .cmp(&a_exact)
            .then(b.downloads.cmp(&a.downloads))
    });

    kept
}

/// Normalize a title for comparison: lowercase, remove special chars, trim.
fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Jaro-Winkler string similarity (0.0 to 1.0).
fn jaro_winkler(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let len1 = s1.len();
    let len2 = s2.len();
    let match_distance = (len1.max(len2) / 2).saturating_sub(1);

    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];
    let mut matches = 0usize;
    let mut transpositions = 0usize;

    let s1_bytes = s1.as_bytes();
    let s2_bytes = s2.as_bytes();

    for i in 0..len1 {
        let start = if i >= match_distance { i - match_distance } else { 0 };
        let end = (i + match_distance + 1).min(len2);

        for j in start..end {
            if s2_matches[j] || s1_bytes[i] != s2_bytes[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_bytes[i] != s2_bytes[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let m = matches as f64;
    let jaro = (m / len1 as f64 + m / len2 as f64 + (m - transpositions as f64 / 2.0) / m) / 3.0;

    // Winkler modification: boost for common prefix
    let prefix = s1_bytes
        .iter()
        .zip(s2_bytes.iter())
        .take(4)
        .take_while(|(a, b)| a == b)
        .count();

    jaro + (prefix as f64 * 0.1 * (1.0 - jaro))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title() {
        assert_eq!(normalize_title("Sodium [Fabric]"), "sodium fabric");
        assert_eq!(normalize_title("OptiFine 1.20.1"), "optifine 1201");
    }

    #[test]
    fn test_jaro_winkler_identical() {
        assert!((jaro_winkler("hello", "hello") - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_jaro_winkler_similar() {
        let sim = jaro_winkler("sodium", "sodium fabric");
        assert!(sim > 0.8);
    }

    #[test]
    fn test_jaro_winkler_different() {
        let sim = jaro_winkler("sodium", "iris");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_dedup_removes_similar() {
        let results = vec![
            AggregatedResult {
                source: "modrinth".into(),
                project_id: "1".into(),
                slug: "sodium".into(),
                title: "Sodium".into(),
                description: "desc".into(),
                icon_url: "".into(),
                downloads: 1000,
                categories: vec![],
                client_side: None,
                server_side: None,
            },
            AggregatedResult {
                source: "curseforge".into(),
                project_id: "2".into(),
                slug: "sodium-fabric".into(),
                title: "Sodium".into(),
                description: "desc".into(),
                icon_url: "".into(),
                downloads: 500,
                categories: vec![],
                client_side: None,
                server_side: None,
            },
            AggregatedResult {
                source: "modrinth".into(),
                project_id: "3".into(),
                slug: "iris".into(),
                title: "Iris Shaders".into(),
                description: "desc".into(),
                icon_url: "".into(),
                downloads: 800,
                categories: vec![],
                client_side: None,
                server_side: None,
            },
        ];

        let deduped = deduplicate_results(results, "sodium");
        assert_eq!(deduped.len(), 2); // Sodium (kept higher downloads) + Iris
        assert_eq!(deduped[0].title, "Sodium");
        assert_eq!(deduped[0].downloads, 1000); // Higher download one kept
    }
}
