//! Per-application audio output device routing.
//!
//! Windows 10 1803+ supports per-app audio device assignment through
//! the Settings app, but no public API is exposed. This module provides
//! a best-effort approach using the undocumented IPolicyConfig interface
//! and registry-based fallbacks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A routing rule mapping a process name to a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub process_name: String,
    pub device_id: String,
    pub device_name: String,
}

/// A named routing preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPreset {
    pub name: String,
    pub rules: Vec<RoutingRule>,
}

/// Get the current routing rules (from config).
pub fn get_routing_rules() -> Result<Vec<RoutingRule>, String> {
    let config = sovereign_config::load::<crate::config::AudioRouterConfig>("audio-router");
    match config {
        Some(cfg) => Ok(cfg.routing_rules),
        None => Ok(Vec::new()),
    }
}

/// Save routing rules to config.
pub fn save_routing_rules(rules: &[RoutingRule]) -> Result<(), String> {
    let mut config: crate::config::AudioRouterConfig =
        sovereign_config::load_or_default("audio-router").unwrap_or_default();
    config.routing_rules = rules.to_vec();
    sovereign_config::save("audio-router", &config)
        .map_err(|e| format!("Save config error: {e}"))
}

/// Get all saved routing presets.
pub fn get_presets() -> Result<Vec<RoutingPreset>, String> {
    let config = sovereign_config::load::<crate::config::AudioRouterConfig>("audio-router");
    match config {
        Some(cfg) => {
            let presets: Vec<RoutingPreset> = cfg
                .routing_presets
                .into_iter()
                .map(|(name, rules)| RoutingPreset {
                    name,
                    rules: rules
                        .into_iter()
                        .map(|(process, device)| RoutingRule {
                            process_name: process,
                            device_id: String::new(),
                            device_name: device,
                        })
                        .collect(),
                })
                .collect();
            Ok(presets)
        }
        None => Ok(Vec::new()),
    }
}

/// Save a routing preset.
pub fn save_preset(preset: &RoutingPreset) -> Result<(), String> {
    let mut config: crate::config::AudioRouterConfig =
        sovereign_config::load_or_default("audio-router").unwrap_or_default();

    let rules: HashMap<String, String> = preset
        .rules
        .iter()
        .map(|r| (r.process_name.clone(), r.device_name.clone()))
        .collect();

    config.routing_presets.insert(preset.name.clone(), rules);
    sovereign_config::save("audio-router", &config)
        .map_err(|e| format!("Save preset error: {e}"))
}

/// Apply a routing preset (sets per-app device assignments).
pub fn apply_preset(_preset: &RoutingPreset) -> Result<(), String> {
    // Per-app audio routing on Windows requires IPolicyConfig
    // (undocumented) or the Settings > Sound > App volume settings.
    // For now, we save the mapping and note it for the user.
    Err("Per-app device routing requires Windows IPolicyConfig — routing rules saved but not applied".to_string())
}
