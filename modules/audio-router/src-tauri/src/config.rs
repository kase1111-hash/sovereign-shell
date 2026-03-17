//! Audio router configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::routing::RoutingRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRouterConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub routing_rules: Vec<RoutingRule>,
    #[serde(default)]
    pub routing_presets: HashMap<String, HashMap<String, String>>,
}

impl Default for AudioRouterConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            routing_rules: Vec::new(),
            routing_presets: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Update interval for level meters in milliseconds.
    pub update_interval_ms: u64,
    /// Whether to show inactive audio sessions.
    pub show_inactive_sessions: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 50,
            show_inactive_sessions: false,
        }
    }
}
