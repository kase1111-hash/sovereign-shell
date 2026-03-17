//! Configuration for the Notification Queue module.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub toast: ToastConfig,
    #[serde(default)]
    pub history: HistoryConfig,
}

impl Default for NotifyConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            toast: ToastConfig::default(),
            history: HistoryConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Maximum notifications in the queue.
    #[serde(default = "default_max_queue")]
    pub max_queue_size: usize,
    /// Start in silent mode.
    #[serde(default)]
    pub silent_mode: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 500,
            silent_mode: false,
        }
    }
}

fn default_max_queue() -> usize {
    500
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastConfig {
    /// Toast position on screen.
    #[serde(default = "default_position")]
    pub position: String,
    /// Default toast duration in seconds.
    #[serde(default = "default_duration")]
    pub default_duration_seconds: u32,
    /// Maximum toasts visible at once.
    #[serde(default = "default_max_visible")]
    pub max_visible: u32,
}

impl Default for ToastConfig {
    fn default() -> Self {
        Self {
            position: "bottom-right".to_string(),
            default_duration_seconds: 5,
            max_visible: 3,
        }
    }
}

fn default_position() -> String {
    "bottom-right".to_string()
}

fn default_duration() -> u32 {
    5
}

fn default_max_visible() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Retention period in days.
    #[serde(default = "default_retention")]
    pub retention_days: u32,
    /// Purge interval in hours.
    #[serde(default = "default_purge_interval")]
    pub purge_interval_hours: u32,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            retention_days: 30,
            purge_interval_hours: 24,
        }
    }
}

fn default_retention() -> u32 {
    30
}

fn default_purge_interval() -> u32 {
    24
}
