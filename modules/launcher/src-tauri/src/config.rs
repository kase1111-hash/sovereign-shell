//! Launcher configuration.
//!
//! Defines the typed config struct that gets loaded from
//! `%APPDATA%\SovereignShell\launcher\config.toml`.

use serde::{Deserialize, Serialize};

/// Top-level launcher configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub hotkey: HotkeyConfig,
    pub indexing: IndexingConfig,
    pub appearance: AppearanceConfig,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            hotkey: HotkeyConfig::default(),
            indexing: IndexingConfig::default(),
            appearance: AppearanceConfig::default(),
        }
    }
}

/// Hotkey activation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifier: String,
    pub key: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifier: "alt".to_string(),
            key: "space".to_string(),
        }
    }
}

/// Indexing behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub refresh_interval_seconds: u64,
    pub max_results: usize,
    pub extra_dirs: Vec<String>,
}

impl Default for IndexingConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 300,
            max_results: 8,
            extra_dirs: Vec::new(),
        }
    }
}

/// Visual appearance settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub window_width: u32,
    pub max_visible_results: usize,
    pub enable_vibrancy: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            window_width: 600,
            max_visible_results: 8,
            enable_vibrancy: true,
        }
    }
}
