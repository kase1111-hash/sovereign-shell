//! Filesystem crawler and content extractor for the search daemon.
//!
//! Recursively walks configured directories, extracts file metadata and
//! optionally text content, and upserts entries into the SQLite index.

use crate::config::SearchDaemonConfig;
use crate::db::Database;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

/// Statistics from an indexing run.
#[derive(Debug, Default, serde::Serialize)]
pub struct CrawlStats {
    pub files_indexed: usize,
    pub files_content_indexed: usize,
    pub files_skipped: usize,
    pub files_pruned: usize,
    pub errors: usize,
    pub total_in_index: i64,
    pub elapsed_ms: u128,
}

/// Run a full index crawl over all configured watched directories.
pub fn run_full_crawl(db: &Database, config: &SearchDaemonConfig) -> Result<CrawlStats, String> {
    let start = std::time::Instant::now();
    let mut stats = CrawlStats::default();

    let crawl_stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let exclude_set: HashSet<String> = config
        .indexing
        .exclude_patterns
        .iter()
        .map(|p| p.to_lowercase())
        .collect();

    let content_exts: HashSet<String> = config
        .indexing
        .content_index_extensions
        .iter()
        .map(|e| e.to_lowercase())
        .collect();

    let max_content_bytes = config.indexing.max_content_size_mb * 1024 * 1024;
    let batch_size = config.performance.batch_size;
    let throttle_ms = config.performance.index_throttle_ms;

    let mut batch_count = 0;

    for watched_dir in &config.indexing.watched_dirs {
        let root = PathBuf::from(watched_dir);
        if !root.exists() {
            warn!("Watched directory does not exist: {}", watched_dir);
            continue;
        }

        info!("Crawling: {}", watched_dir);

        // Start a batch transaction
        db.begin_batch().map_err(|e| format!("DB begin error: {e}"))?;

        let walker = WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !is_excluded(entry.path(), &exclude_set));

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    debug!("Walk error: {}", e);
                    stats.errors += 1;
                    continue;
                }
            };

            // Skip directories — we only index files
            if entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    debug!("Metadata error for {}: {}", path.display(), e);
                    stats.errors += 1;
                    continue;
                }
            };

            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let extension = path
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()));

            let parent_dir = path
                .parent()
                .and_then(|p| p.file_name())
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let size = metadata.len() as i64;
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let path_str = path.to_string_lossy().to_string();

            // Extract content for supported file types
            let content = if should_index_content(&extension, &content_exts, size as u64, max_content_bytes) {
                match extract_text_content(path) {
                    Ok(text) => {
                        stats.files_content_indexed += 1;
                        Some(text)
                    }
                    Err(e) => {
                        debug!("Content extraction failed for {}: {}", path_str, e);
                        None
                    }
                }
            } else {
                None
            };

            match db.upsert_file(
                &path_str,
                &file_name,
                extension.as_deref(),
                size,
                modified,
                &parent_dir,
                content.as_deref(),
                crawl_stamp,
            ) {
                Ok(()) => stats.files_indexed += 1,
                Err(e) => {
                    debug!("Index error for {}: {}", path_str, e);
                    stats.errors += 1;
                }
            }

            // Batch commit for performance
            batch_count += 1;
            if batch_count >= batch_size {
                db.commit_batch().map_err(|e| format!("DB commit error: {e}"))?;
                batch_count = 0;

                // Throttle to avoid disk thrash
                if throttle_ms > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(throttle_ms));
                }

                db.begin_batch().map_err(|e| format!("DB begin error: {e}"))?;
            }
        }

        // Commit any remaining batch
        db.commit_batch().map_err(|e| format!("DB commit error: {e}"))?;
    }

    // Prune files that no longer exist
    match db.prune_unseen(crawl_stamp) {
        Ok(n) => {
            stats.files_pruned = n;
            if n > 0 {
                info!("Pruned {} stale entries", n);
            }
        }
        Err(e) => warn!("Prune failed: {}", e),
    }

    stats.total_in_index = db.count().unwrap_or(0);
    stats.elapsed_ms = start.elapsed().as_millis();

    Ok(stats)
}

/// Index a single file (used for incremental updates from the file watcher).
pub fn index_single_file(
    db: &Database,
    path: &Path,
    config: &SearchDaemonConfig,
) -> Result<(), String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Metadata error: {e}"))?;

    if metadata.is_dir() {
        return Ok(());
    }

    let content_exts: HashSet<String> = config
        .indexing
        .content_index_extensions
        .iter()
        .map(|e| e.to_lowercase())
        .collect();
    let max_content_bytes = config.indexing.max_content_size_mb * 1024 * 1024;

    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let extension = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()));

    let parent_dir = path
        .parent()
        .and_then(|p| p.file_name())
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let size = metadata.len() as i64;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let crawl_stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let content = if should_index_content(&extension, &content_exts, size as u64, max_content_bytes) {
        extract_text_content(path).ok()
    } else {
        None
    };

    db.upsert_file(
        &path.to_string_lossy(),
        &file_name,
        extension.as_deref(),
        size,
        modified,
        &parent_dir,
        content.as_deref(),
        crawl_stamp,
    )
    .map_err(|e| format!("DB error: {e}"))
}

/// Check if a directory entry should be excluded based on patterns.
fn is_excluded(path: &Path, exclude_set: &HashSet<String>) -> bool {
    if let Some(name) = path.file_name() {
        let name_lower = name.to_string_lossy().to_lowercase();

        // Check direct name match
        if exclude_set.contains(&name_lower) {
            return true;
        }

        // Check if any exclude pattern appears as a path component
        let path_str = path.to_string_lossy().to_lowercase();
        for pattern in exclude_set {
            if pattern.contains('\\') || pattern.contains('/') {
                // Path-based pattern
                if path_str.contains(&pattern.to_lowercase()) {
                    return true;
                }
            }
        }
    }
    false
}

/// Determine if content extraction should be attempted for this file.
fn should_index_content(
    extension: &Option<String>,
    content_exts: &HashSet<String>,
    size: u64,
    max_bytes: u64,
) -> bool {
    if size > max_bytes || size == 0 {
        return false;
    }
    match extension {
        Some(ext) => content_exts.contains(&ext.to_lowercase()),
        None => false,
    }
}

/// Extract text content from a file for indexing.
/// Handles plain text files and HTML (strip tags). Returns up to 100KB of text.
fn extract_text_content(path: &Path) -> Result<String, String> {
    let ext = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    let raw = std::fs::read_to_string(path)
        .map_err(|e| format!("Read error: {e}"))?;

    let text = match ext.as_str() {
        "html" | "htm" => strip_html_tags(&raw),
        _ => raw,
    };

    // Truncate to 100KB to keep the FTS index manageable
    let max_len = 100 * 1024;
    if text.len() > max_len {
        Ok(text[..max_len].to_string())
    } else {
        Ok(text)
    }
}

/// Simple HTML tag stripper. Removes tags and decodes basic entities.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                result.push(' ');
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}
