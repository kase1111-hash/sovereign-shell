//! Explorer configuration types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub sidebar: SidebarConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            sidebar: SidebarConfig::default(),
            appearance: AppearanceConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub default_view: String,
    pub show_hidden_files: bool,
    pub show_file_extensions: bool,
    pub confirm_delete: bool,
    pub terminal_follows_navigation: bool,
    pub default_shell: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_view: "list".to_string(),
            show_hidden_files: false,
            show_file_extensions: true,
            confirm_delete: true,
            terminal_follows_navigation: true,
            #[cfg(windows)]
            default_shell: "powershell".to_string(),
            #[cfg(not(windows))]
            default_shell: "bash".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarConfig {
    pub show_drives: bool,
    pub bookmarks: Vec<String>,
}

impl Default for SidebarConfig {
    fn default() -> Self {
        Self {
            show_drives: true,
            bookmarks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub icon_size_grid: u32,
    pub preview_pane_default: bool,
    pub terminal_pane_default: bool,
    pub enable_vibrancy: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            icon_size_grid: 48,
            preview_pane_default: false,
            terminal_pane_default: false,
            enable_vibrancy: true,
        }
    }
}
