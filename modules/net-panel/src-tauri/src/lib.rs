//! Sovereign Shell — Network Panel module.

mod adapters;
mod config;
mod connections;
mod diagnostics;
mod dns;
mod firewall;
mod monitor;

use config::NetPanelConfig;
use monitor::BandwidthSnapshot;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    config: Mutex<NetPanelConfig>,
    last_bandwidth: Mutex<Option<BandwidthSnapshot>>,
}

// ── Adapter commands ──

#[tauri::command]
fn get_adapters() -> Result<Vec<adapters::NetworkAdapter>, String> {
    adapters::enumerate_adapters()
}

#[tauri::command]
fn set_adapter_state(name: String, enabled: bool) -> Result<(), String> {
    adapters::set_adapter_state(&name, enabled)
}

#[tauri::command]
fn set_dns_servers(adapter_name: String, servers: Vec<String>) -> Result<(), String> {
    adapters::set_dns(&adapter_name, &servers)
}

// ── Connection commands ──

#[tauri::command]
fn get_connections() -> Result<Vec<connections::ConnectionEntry>, String> {
    connections::get_connections()
}

// ── Firewall commands ──

#[tauri::command]
fn get_firewall_rules() -> Result<Vec<firewall::FirewallRule>, String> {
    firewall::get_rules()
}

#[tauri::command]
fn toggle_firewall_rule(name: String, enabled: bool) -> Result<(), String> {
    firewall::toggle_rule(&name, enabled)
}

#[tauri::command]
fn create_firewall_rule(rule: firewall::FirewallRule) -> Result<(), String> {
    firewall::create_rule(&rule)
}

// ── Diagnostics commands ──

#[tauri::command]
fn run_ping(host: String, count: u32) -> Result<Vec<diagnostics::PingResult>, String> {
    diagnostics::ping(&host, count)
}

#[tauri::command]
fn run_traceroute(host: String, max_hops: u32) -> Result<Vec<diagnostics::TracerouteHop>, String> {
    diagnostics::traceroute(&host, max_hops)
}

// ── DNS commands ──

#[tauri::command]
fn dns_lookup(
    host: String,
    record_type: String,
    server: Option<String>,
) -> Result<dns::DnsResult, String> {
    dns::lookup(&host, &record_type, server.as_deref())
}

// ── Bandwidth commands ──

#[tauri::command]
fn get_bandwidth(state: State<'_, AppState>) -> Result<BandwidthSnapshot, String> {
    let prev = state.last_bandwidth.lock().unwrap().clone();
    let snapshot = monitor::collect_bandwidth(&prev);
    *state.last_bandwidth.lock().unwrap() = Some(snapshot.clone());
    Ok(snapshot)
}

// ── Config commands ──

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<NetPanelConfig, String> {
    Ok(state.config.lock().unwrap().clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(NetPanelConfig::default()),
            last_bandwidth: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_adapters,
            set_adapter_state,
            set_dns_servers,
            get_connections,
            get_firewall_rules,
            toggle_firewall_rule,
            create_firewall_rule,
            run_ping,
            run_traceroute,
            dns_lookup,
            get_bandwidth,
            get_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running net-panel");
}
