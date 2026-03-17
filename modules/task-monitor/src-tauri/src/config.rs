//! Task monitor configuration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMonitorConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub tray: TrayConfig,
}

impl Default for TaskMonitorConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            tray: TrayConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub update_interval_ms: u64,
    pub default_view: String,
    pub show_system_processes: bool,
    pub confirm_kill: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1000,
            default_view: "processes".to_string(),
            show_system_processes: false,
            confirm_kill: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayConfig {
    pub minimize_to_tray: bool,
    pub show_cpu_indicator: bool,
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self {
            minimize_to_tray: true,
            show_cpu_indicator: true,
        }
    }
}
