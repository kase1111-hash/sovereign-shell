//! Sovereign Task Monitor — Tauri command layer.

pub mod config;
pub mod file_locks;
pub mod process_actions;
pub mod processes;
pub mod services;
pub mod system_stats;

use config::TaskMonitorConfig;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};
use std::sync::Mutex;
use tauri::State;

/// Shared application state.
pub struct AppState {
    pub sys: Mutex<System>,
    pub config: TaskMonitorConfig,
}

// ── Tauri Commands ──────────────────────────────────────────────────

#[tauri::command]
fn get_processes(state: State<'_, AppState>) -> Result<Vec<processes::ProcessInfo>, String> {
    let mut sys = state.sys.lock().map_err(|e| format!("{e}"))?;
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );
    Ok(processes::enumerate(&sys))
}

#[tauri::command]
fn get_process_tree(state: State<'_, AppState>) -> Result<Vec<processes::ProcessTreeNode>, String> {
    let mut sys = state.sys.lock().map_err(|e| format!("{e}"))?;
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );
    let procs = processes::enumerate(&sys);
    Ok(processes::build_tree(&procs))
}

#[tauri::command]
fn get_system_stats(state: State<'_, AppState>) -> Result<system_stats::SystemStats, String> {
    let mut sys = state.sys.lock().map_err(|e| format!("{e}"))?;
    sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    sys.refresh_memory_specifics(MemoryRefreshKind::everything());
    Ok(system_stats::collect(&sys))
}

#[tauri::command]
fn kill_process(state: State<'_, AppState>, pid: u32) -> Result<(), String> {
    let sys = state.sys.lock().map_err(|e| format!("{e}"))?;
    process_actions::kill_process(&sys, pid)
}

#[tauri::command]
fn kill_process_tree(state: State<'_, AppState>, pid: u32) -> Result<usize, String> {
    let sys = state.sys.lock().map_err(|e| format!("{e}"))?;
    process_actions::kill_tree(&sys, pid)
}

#[tauri::command]
fn suspend_process(pid: u32) -> Result<(), String> {
    process_actions::suspend_process(pid)
}

#[tauri::command]
fn resume_process(pid: u32) -> Result<(), String> {
    process_actions::resume_process(pid)
}

#[tauri::command]
fn set_process_priority(pid: u32, priority: String) -> Result<(), String> {
    process_actions::set_priority(pid, &priority)
}

#[tauri::command]
fn find_file_locks(file_path: String) -> Result<Vec<file_locks::LockingProcess>, String> {
    file_locks::find_locking_processes(&file_path)
}

#[tauri::command]
fn get_services() -> Result<Vec<services::ServiceInfo>, String> {
    services::enumerate_services()
}

#[tauri::command]
fn start_service(name: String) -> Result<(), String> {
    services::start_service(&name)
}

#[tauri::command]
fn stop_service(name: String) -> Result<(), String> {
    services::stop_service(&name)
}

#[tauri::command]
fn restart_service(name: String) -> Result<(), String> {
    services::restart_service(&name)
}

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<TaskMonitorConfig, String> {
    Ok(state.config.clone())
}

/// Build and run the Tauri application.
pub fn run() {
    let config: TaskMonitorConfig =
        sovereign_config::load_or_default("task-monitor").unwrap_or_default();

    let sys = System::new_all();

    let state = AppState {
        sys: Mutex::new(sys),
        config,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_processes,
            get_process_tree,
            get_system_stats,
            kill_process,
            kill_process_tree,
            suspend_process,
            resume_process,
            set_process_priority,
            find_file_locks,
            get_services,
            start_service,
            stop_service,
            restart_service,
            get_config,
        ])
        .setup(|app| {
            #[cfg(windows)]
            {
                let window = app.get_webview_window("main").unwrap();
                let _ = window_vibrancy::apply_acrylic(&window, Some((18, 18, 26, 200)));
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running sovereign-task-monitor");
}
