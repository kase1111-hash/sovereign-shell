//! Search daemon configuration.
//!
//! Loaded from `%APPDATA%\SovereignShell\search-daemon\config.toml`.

use serde::{Deserialize, Serialize};

/// Top-level search daemon configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDaemonConfig {
    pub indexing: IndexingConfig,
    pub performance: PerformanceConfig,
    pub ipc: IpcConfig,
}

impl Default for SearchDaemonConfig {
    fn default() -> Self {
        Self {
            indexing: IndexingConfig::default(),
            performance: PerformanceConfig::default(),
            ipc: IpcConfig::default(),
        }
    }
}

/// Directories and rules for indexing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    /// Root directories to watch and index.
    pub watched_dirs: Vec<String>,
    /// Directory name patterns to exclude from crawling.
    pub exclude_patterns: Vec<String>,
    /// File extensions for which text content should be extracted and indexed.
    pub content_index_extensions: Vec<String>,
    /// Maximum file size in MB for content extraction.
    pub max_content_size_mb: u64,
    /// Hours between full re-index passes.
    pub reindex_interval_hours: u64,
}

impl Default for IndexingConfig {
    fn default() -> Self {
        Self {
            watched_dirs: default_watched_dirs(),
            exclude_patterns: vec![
                "node_modules".into(),
                ".git".into(),
                "target".into(),
                "__pycache__".into(),
                ".cache".into(),
                "AppData\\Local\\Temp".into(),
            ],
            content_index_extensions: vec![
                ".txt".into(), ".md".into(), ".log".into(), ".csv".into(),
                ".html".into(), ".htm".into(),
                ".rs".into(), ".py".into(), ".js".into(), ".ts".into(),
                ".toml".into(), ".json".into(), ".yaml".into(), ".yml".into(),
                ".ps1".into(), ".bat".into(), ".sh".into(),
                ".c".into(), ".cpp".into(), ".h".into(), ".hpp".into(),
                ".java".into(), ".go".into(), ".rb".into(),
            ],
            max_content_size_mb: 10,
            reindex_interval_hours: 24,
        }
    }
}

/// Performance tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of files to index per batch before yielding.
    pub batch_size: usize,
    /// Milliseconds to pause between batches.
    pub index_throttle_ms: u64,
    /// Soft memory limit in MB — pause indexing if exceeded.
    pub max_memory_mb: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            batch_size: 500,
            index_throttle_ms: 10,
            max_memory_mb: 256,
        }
    }
}

/// IPC server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcConfig {
    /// Named pipe name (used as module name for sovereign_ipc).
    pub pipe_name: String,
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            pipe_name: "search-daemon".into(),
        }
    }
}

/// Returns platform-appropriate default watched directories.
fn default_watched_dirs() -> Vec<String> {
    #[cfg(windows)]
    {
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            vec![profile.to_string_lossy().to_string()]
        } else {
            vec!["C:\\Users".to_string()]
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            vec![home.to_string_lossy().to_string()]
        } else {
            vec!["/home".to_string()]
        }
    }
}
