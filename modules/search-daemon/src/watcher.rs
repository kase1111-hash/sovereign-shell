//! Filesystem watcher for incremental index updates.
//!
//! Uses the `notify` crate (backed by ReadDirectoryChangesW on Windows)
//! to detect file create/modify/delete/rename events and update the index.

use crate::config::SearchDaemonConfig;
use crate::db::Database;
use crate::indexer;
use log::{debug, info, warn};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Start watching configured directories for changes.
/// Returns a handle that keeps the watcher alive.
/// Changes are debounced (500ms) and applied to the database.
pub fn start_watching(
    db: Arc<Mutex<Database>>,
    config: Arc<SearchDaemonConfig>,
) -> Result<RecommendedWatcher, String> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        notify::Config::default()
            .with_poll_interval(Duration::from_secs(2)),
    )
    .map_err(|e| format!("Failed to create watcher: {e}"))?;

    // Watch all configured directories
    for dir in &config.indexing.watched_dirs {
        let path = PathBuf::from(dir);
        if path.exists() {
            match watcher.watch(&path, RecursiveMode::Recursive) {
                Ok(()) => info!("Watching: {}", dir),
                Err(e) => warn!("Failed to watch {}: {}", dir, e),
            }
        }
    }

    // Build exclusion set
    let exclude_set: HashSet<String> = config
        .indexing
        .exclude_patterns
        .iter()
        .map(|p| p.to_lowercase())
        .collect();

    // Spawn a thread to process debounced events
    let config_clone = config.clone();
    std::thread::spawn(move || {
        let mut pending: HashSet<PathBuf> = HashSet::new();
        let mut pending_removes: HashSet<PathBuf> = HashSet::new();
        let mut last_event = Instant::now();
        let debounce = Duration::from_millis(500);

        loop {
            match rx.recv_timeout(Duration::from_millis(200)) {
                Ok(Ok(event)) => {
                    for path in &event.paths {
                        // Skip excluded paths
                        if is_excluded_path(path, &exclude_set) {
                            continue;
                        }

                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                pending_removes.remove(path);
                                pending.insert(path.clone());
                            }
                            EventKind::Remove(_) => {
                                pending.remove(path);
                                pending_removes.insert(path.clone());
                            }
                            _ => {}
                        }
                    }
                    last_event = Instant::now();
                }
                Ok(Err(e)) => {
                    debug!("Watch error: {}", e);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if debounce period has elapsed
                    if last_event.elapsed() >= debounce
                        && (!pending.is_empty() || !pending_removes.is_empty())
                    {
                        process_changes(
                            &db,
                            &config_clone,
                            &mut pending,
                            &mut pending_removes,
                        );
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    info!("Watcher channel disconnected, stopping");
                    break;
                }
            }
        }
    });

    Ok(watcher)
}

/// Process accumulated file changes.
fn process_changes(
    db: &Arc<Mutex<Database>>,
    config: &SearchDaemonConfig,
    pending: &mut HashSet<PathBuf>,
    pending_removes: &mut HashSet<PathBuf>,
) {
    let db = match db.lock() {
        Ok(db) => db,
        Err(e) => {
            warn!("DB lock error in watcher: {}", e);
            return;
        }
    };

    // Process removals
    for path in pending_removes.drain() {
        let path_str = path.to_string_lossy();
        if let Err(e) = db.remove_file(&path_str) {
            debug!("Remove error for {}: {}", path_str, e);
        } else {
            debug!("Removed from index: {}", path_str);
        }
    }

    // Process creates/modifies
    let mut indexed = 0;
    for path in pending.drain() {
        if !path.exists() || path.is_dir() {
            continue;
        }
        match indexer::index_single_file(&db, &path, config) {
            Ok(()) => indexed += 1,
            Err(e) => debug!("Index error for {}: {}", path.display(), e),
        }
    }

    if indexed > 0 {
        info!("Watcher: indexed {} changed files", indexed);
    }
}

/// Check if a path should be excluded from watching events.
fn is_excluded_path(path: &PathBuf, exclude_set: &HashSet<String>) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    for pattern in exclude_set {
        let pattern_lower = pattern.to_lowercase();
        // Check if any path component matches
        if path_str.contains(&format!("{}{}",
            std::path::MAIN_SEPARATOR, pattern_lower))
            || path_str.contains(&format!("{}/", pattern_lower))
            || path_str.contains(&format!("{}\\", pattern_lower))
        {
            return true;
        }
    }
    false
}
