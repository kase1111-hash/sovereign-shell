//! IPC server for the search daemon.
//!
//! Exposes search, status, and reindex commands over a Windows named pipe
//! using the shared sovereign_ipc crate.

use crate::config::SearchDaemonConfig;
use crate::db::Database;
use log::{info, warn};
use sovereign_ipc::Message;
use std::sync::{Arc, Mutex};

/// Request payload for search queries.
#[derive(Debug, serde::Deserialize)]
struct SearchRequest {
    query: String,
    #[serde(default = "default_max_results")]
    max_results: usize,
    #[serde(default)]
    file_types: Vec<String>,
}

fn default_max_results() -> usize { 20 }

/// Request payload for reindex commands.
#[derive(Debug, serde::Deserialize)]
struct ReindexRequest {
    #[serde(default)]
    path: Option<String>,
}

/// Start the IPC server in a background thread.
/// This blocks forever — call from a dedicated thread.
pub fn start_ipc_server(
    db: Arc<Mutex<Database>>,
    config: Arc<SearchDaemonConfig>,
) {
    let pipe_name = config.ipc.pipe_name.clone();
    info!("Starting IPC server on pipe: {}", pipe_name);

    let server_config = sovereign_ipc::server::ServerConfig::new(&pipe_name);

    let handler: sovereign_ipc::server::Handler = Box::new(move |msg: Message| {
        match msg.msg_type.as_str() {
            "search" => handle_search(&db, &msg),
            "status" => handle_status(&db),
            "reindex" => handle_reindex(&msg),
            other => Message::error(&format!("Unknown message type: {}", other)),
        }
    });

    if let Err(e) = sovereign_ipc::server::run(&server_config, &handler) {
        warn!("IPC server error: {}", e);
    }
}

/// Handle a search request.
fn handle_search(db: &Arc<Mutex<Database>>, msg: &Message) -> Message {
    let request: SearchRequest = match msg.parse_payload() {
        Ok(r) => r,
        Err(e) => return Message::error(&format!("Invalid search request: {e}")),
    };

    let db = match db.lock() {
        Ok(db) => db,
        Err(e) => return Message::error(&format!("DB lock error: {e}")),
    };

    let start = std::time::Instant::now();
    let file_types = if request.file_types.is_empty() {
        None
    } else {
        Some(request.file_types.as_slice())
    };

    match db.search(&request.query, request.max_results, file_types) {
        Ok(hits) => {
            let query_ms = start.elapsed().as_millis();
            let total = hits.len();
            match Message::new("results", &serde_json::json!({
                "hits": hits,
                "total": total,
                "query_ms": query_ms,
            })) {
                Ok(msg) => msg,
                Err(e) => Message::error(&format!("Serialize error: {e}")),
            }
        }
        Err(e) => Message::error(&format!("Search error: {e}")),
    }
}

/// Handle a status request.
fn handle_status(db: &Arc<Mutex<Database>>) -> Message {
    let db = match db.lock() {
        Ok(db) => db,
        Err(e) => return Message::error(&format!("DB lock error: {e}")),
    };

    match db.status() {
        Ok(status) => match Message::new("status", &status) {
            Ok(msg) => msg,
            Err(e) => Message::error(&format!("Serialize error: {e}")),
        },
        Err(e) => Message::error(&format!("Status error: {e}")),
    }
}

/// Handle a reindex request (returns acknowledgment; actual reindex is async).
fn handle_reindex(msg: &Message) -> Message {
    let _request: ReindexRequest = match msg.parse_payload() {
        Ok(r) => r,
        Err(_) => ReindexRequest { path: None },
    };

    // TODO: Signal the main indexing thread to re-run.
    // For now, acknowledge the request.
    match Message::new("ack", &serde_json::json!({
        "message": "Reindex scheduled"
    })) {
        Ok(msg) => msg,
        Err(e) => Message::error(&format!("Serialize error: {e}")),
    }
}
