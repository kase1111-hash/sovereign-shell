//! Sovereign Audio Router — Tauri command layer.

pub mod config;
pub mod devices;
pub mod events;
pub mod monitor;
pub mod routing;
pub mod sessions;
pub mod volume;

use config::AudioRouterConfig;
use std::sync::Mutex;
use tauri::State;

/// Shared application state.
pub struct AppState {
    pub config: AudioRouterConfig,
    /// Track previous device/session IDs for change detection.
    pub prev_device_ids: Mutex<Vec<String>>,
    pub prev_session_pids: Mutex<Vec<u32>>,
}

// ── Tauri Commands ──────────────────────────────────────────────────

#[tauri::command]
fn get_devices() -> Result<Vec<devices::AudioDevice>, String> {
    devices::enumerate_devices()
}

#[tauri::command]
fn get_sessions() -> Result<Vec<sessions::AudioSession>, String> {
    sessions::enumerate_sessions()
}

#[tauri::command]
fn set_device_volume(device_id: String, level: f32) -> Result<(), String> {
    volume::set_device_volume(&device_id, level)
}

#[tauri::command]
fn set_device_mute(device_id: String, muted: bool) -> Result<(), String> {
    volume::set_device_mute(&device_id, muted)
}

#[tauri::command]
fn set_session_volume(pid: u32, level: f32) -> Result<(), String> {
    volume::set_session_volume(pid, level)
}

#[tauri::command]
fn set_session_mute(pid: u32, muted: bool) -> Result<(), String> {
    volume::set_session_mute(pid, muted)
}

#[tauri::command]
fn get_peak_levels() -> Result<monitor::LevelSnapshot, String> {
    monitor::get_peak_levels()
}

#[tauri::command]
fn get_routing_rules() -> Result<Vec<routing::RoutingRule>, String> {
    routing::get_routing_rules()
}

#[tauri::command]
fn save_routing_rules(rules: Vec<routing::RoutingRule>) -> Result<(), String> {
    routing::save_routing_rules(&rules)
}

#[tauri::command]
fn get_presets() -> Result<Vec<routing::RoutingPreset>, String> {
    routing::get_presets()
}

#[tauri::command]
fn save_preset(preset: routing::RoutingPreset) -> Result<(), String> {
    routing::save_preset(&preset)
}

#[tauri::command]
fn apply_preset(preset: routing::RoutingPreset) -> Result<(), String> {
    routing::apply_preset(&preset)
}

#[tauri::command]
fn poll_audio_events(state: State<'_, AppState>) -> Result<Vec<events::AudioEvent>, String> {
    let prev_devices = state.prev_device_ids.lock().map_err(|e| format!("{e}"))?;
    let prev_sessions = state.prev_session_pids.lock().map_err(|e| format!("{e}"))?;

    let (evts, new_devices, new_sessions) =
        events::poll_changes(&prev_devices, &prev_sessions)?;

    drop(prev_devices);
    drop(prev_sessions);

    // Update tracked state
    *state.prev_device_ids.lock().map_err(|e| format!("{e}"))? = new_devices;
    *state.prev_session_pids.lock().map_err(|e| format!("{e}"))? = new_sessions;

    Ok(evts)
}

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<AudioRouterConfig, String> {
    Ok(state.config.clone())
}

/// Build and run the Tauri application.
pub fn run() {
    let config: AudioRouterConfig =
        sovereign_config::load_or_default("audio-router").unwrap_or_default();

    let state = AppState {
        config,
        prev_device_ids: Mutex::new(Vec::new()),
        prev_session_pids: Mutex::new(Vec::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_devices,
            get_sessions,
            set_device_volume,
            set_device_mute,
            set_session_volume,
            set_session_mute,
            get_peak_levels,
            get_routing_rules,
            save_routing_rules,
            get_presets,
            save_preset,
            apply_preset,
            poll_audio_events,
            get_config,
        ])
        .setup(|app| {
            #[cfg(windows)]
            {
                let window = app.get_webview_window("main").unwrap();
                let _ = window_vibrancy::apply_acrylic(&window, Some((18, 18, 26, 220)));
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running sovereign-audio-router");
}
