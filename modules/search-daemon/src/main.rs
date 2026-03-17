//! Sovereign Search Daemon — Main Entry Point
//!
//! A headless background service that indexes files on disk and serves
//! search queries over a named pipe IPC interface.
//! Replaces Windows Search (WSearch) with a local, transparent alternative.

mod config;
mod db;
mod indexer;
mod ipc_server;
mod watcher;

use config::SearchDaemonConfig;
use db::Database;
use log::{error, info};
use std::sync::{Arc, Mutex};

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format_timestamp_millis()
    .init();

    info!("Sovereign Search Daemon starting");

    // Load config
    let config: SearchDaemonConfig =
        sovereign_config::load_or_default("search-daemon").unwrap_or_default();
    let config = Arc::new(config);

    info!("Watching {} directories", config.indexing.watched_dirs.len());
    for dir in &config.indexing.watched_dirs {
        info!("  - {}", dir);
    }

    // Open database
    let db = match Database::open_default() {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    let db = Arc::new(Mutex::new(db));

    // Run initial full index
    info!("Running initial full index...");
    {
        let db_lock = db.lock().expect("DB lock failed");
        match indexer::run_full_crawl(&db_lock, &config) {
            Ok(stats) => {
                info!(
                    "Initial index complete: {} files ({} with content), {} pruned, {} errors, {} total in index — took {}ms",
                    stats.files_indexed,
                    stats.files_content_indexed,
                    stats.files_pruned,
                    stats.errors,
                    stats.total_in_index,
                    stats.elapsed_ms,
                );
            }
            Err(e) => {
                error!("Initial index failed: {}", e);
            }
        }
    }

    // Start file watcher for incremental updates
    let _watcher = match watcher::start_watching(db.clone(), config.clone()) {
        Ok(w) => {
            info!("File watcher started");
            Some(w)
        }
        Err(e) => {
            error!("Failed to start file watcher: {}", e);
            None
        }
    };

    // Start periodic re-index thread
    let db_reindex = db.clone();
    let config_reindex = config.clone();
    let reindex_hours = config.indexing.reindex_interval_hours;
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(reindex_hours * 3600));
            info!("Running scheduled re-index...");
            let db_lock = match db_reindex.lock() {
                Ok(db) => db,
                Err(e) => {
                    error!("DB lock failed for re-index: {}", e);
                    continue;
                }
            };
            match indexer::run_full_crawl(&db_lock, &config_reindex) {
                Ok(stats) => {
                    info!("Re-index complete: {} files, {} pruned — took {}ms",
                        stats.files_indexed, stats.files_pruned, stats.elapsed_ms);
                }
                Err(e) => error!("Re-index failed: {}", e),
            }
        }
    });

    // Start IPC server (blocks forever on the main thread)
    info!("Starting IPC server...");
    ipc_server::start_ipc_server(db, config);
}
