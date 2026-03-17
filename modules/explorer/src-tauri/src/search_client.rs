//! IPC client for querying the search daemon.

use serde::Serialize;
use sovereign_ipc::Message;

/// A search result from the daemon.
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub score: f64,
    pub snippet: Option<String>,
    pub size: i64,
    pub modified: i64,
}

/// Query the search daemon for files matching a query.
pub fn search(query: &str, max_results: usize, file_types: &[String]) -> Result<Vec<SearchResult>, String> {
    let payload = serde_json::json!({
        "query": query,
        "max_results": max_results,
        "file_types": file_types,
    });

    let msg = Message::new("search", &payload)
        .map_err(|e| format!("Message create error: {e}"))?;

    let response = sovereign_ipc::client::query("search-daemon", &msg)
        .map_err(|e| format!("Search daemon unavailable: {e}"))?;

    if response.msg_type == "error" {
        return Err(format!("Search error: {}", response.payload));
    }

    // Parse hits from response
    let hits = response.payload.get("hits")
        .and_then(|h| h.as_array())
        .cloned()
        .unwrap_or_default();

    let results: Vec<SearchResult> = hits.iter().filter_map(|h| {
        Some(SearchResult {
            path: h.get("path")?.as_str()?.to_string(),
            name: h.get("name")?.as_str()?.to_string(),
            score: h.get("score")?.as_f64()?,
            snippet: h.get("snippet").and_then(|s| s.as_str()).map(|s| s.to_string()),
            size: h.get("size")?.as_i64()?,
            modified: h.get("modified")?.as_i64()?,
        })
    }).collect();

    Ok(results)
}

/// Check if the search daemon is running.
pub fn is_available() -> bool {
    sovereign_ipc::client::is_available("search-daemon")
}
