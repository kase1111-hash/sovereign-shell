//! Net panel configuration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetPanelConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub dns: DnsConfig,
    #[serde(default)]
    pub diagnostics: DiagnosticsConfig,
}

impl Default for NetPanelConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            dns: DnsConfig::default(),
            diagnostics: DiagnosticsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub update_interval_ms: u64,
    pub show_loopback: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self { update_interval_ms: 1000, show_loopback: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub preferred_servers: Vec<String>,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self { preferred_servers: vec!["1.1.1.1".into(), "9.9.9.9".into()] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsConfig {
    pub default_ping_count: u32,
    pub default_traceroute_max_hops: u32,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self { default_ping_count: 10, default_traceroute_max_hops: 30 }
    }
}
