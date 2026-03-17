//! Sovereign Shell — Notification Queue module.

mod config;
mod history;
mod ipc_server;
mod listener;
mod queue;
mod rules;

use config::NotifyConfig;
use history::{HistoryDb, HistoryEntry};
use queue::{Notification, NotificationGroup, NotificationQueue, Priority};
use rules::{DefaultRule, NotificationRule, RuleAction, RulesEngine};
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};
use tauri::State;

struct AppState {
    queue: Mutex<NotificationQueue>,
    rules: Mutex<RulesEngine>,
    history: Mutex<HistoryDb>,
    config: Mutex<NotifyConfig>,
    silent_mode: Mutex<bool>,
}

fn data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata)
            .join("SovereignShell")
            .join("notify-queue")
    }
    #[cfg(not(windows))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("sovereign-shell")
            .join("notify-queue")
    }
}

// ── Queue commands ──

#[tauri::command]
fn get_notifications(state: State<'_, AppState>) -> Result<Vec<Notification>, String> {
    Ok(state.queue.lock().unwrap().get_all())
}

#[tauri::command]
fn get_grouped_notifications(state: State<'_, AppState>) -> Result<Vec<NotificationGroup>, String> {
    Ok(state.queue.lock().unwrap().get_grouped())
}

#[tauri::command]
fn get_unread_count(state: State<'_, AppState>) -> Result<usize, String> {
    Ok(state.queue.lock().unwrap().unread_count())
}

#[tauri::command]
fn dismiss_notification(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.queue.lock().unwrap().dismiss(&id);
    Ok(())
}

#[tauri::command]
fn dismiss_by_source(source: String, state: State<'_, AppState>) -> Result<(), String> {
    state.queue.lock().unwrap().dismiss_by_source(&source);
    Ok(())
}

#[tauri::command]
fn dismiss_all(state: State<'_, AppState>) -> Result<(), String> {
    state.queue.lock().unwrap().dismiss_all();
    Ok(())
}

#[tauri::command]
fn mark_read(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.queue.lock().unwrap().mark_read(&id);
    Ok(())
}

// ── Send notification (for testing / internal use) ──

#[tauri::command]
fn send_notification(
    source: String,
    title: String,
    body: String,
    priority: Option<String>,
    state: State<'_, AppState>,
) -> Result<Notification, String> {
    let prio = match priority.as_deref() {
        Some("low") => Priority::Low,
        Some("high") => Priority::High,
        Some("critical") => Priority::Critical,
        _ => Priority::Normal,
    };

    let notif = Notification::new(&source, &title, &body, prio);

    // Store in history
    let _ = state.history.lock().unwrap().store(&notif);

    // Add to queue
    state.queue.lock().unwrap().push(notif.clone());

    Ok(notif)
}

// ── Rules commands ──

#[tauri::command]
fn get_rules(state: State<'_, AppState>) -> Result<Vec<NotificationRule>, String> {
    Ok(state.rules.lock().unwrap().get_rules())
}

#[tauri::command]
fn set_rule(rule: NotificationRule, state: State<'_, AppState>) -> Result<(), String> {
    state.rules.lock().unwrap().set_rule(rule);
    Ok(())
}

#[tauri::command]
fn remove_rule(source: String, state: State<'_, AppState>) -> Result<(), String> {
    state.rules.lock().unwrap().remove_rule(&source);
    Ok(())
}

// ── Silent mode ──

#[tauri::command]
fn get_silent_mode(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.silent_mode.lock().unwrap())
}

#[tauri::command]
fn set_silent_mode(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    *state.silent_mode.lock().unwrap() = enabled;
    Ok(())
}

// ── History commands ──

#[tauri::command]
fn search_history(
    query: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<HistoryEntry>, String> {
    state
        .history
        .lock()
        .unwrap()
        .search(&query, limit.unwrap_or(50))
}

#[tauri::command]
fn get_recent_history(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<HistoryEntry>, String> {
    state
        .history
        .lock()
        .unwrap()
        .get_recent(limit.unwrap_or(100))
}

// ── Config ──

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<NotifyConfig, String> {
    Ok(state.config.lock().unwrap().clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);

    let db_path = dir.join("history.db");
    let history_db = HistoryDb::open(&db_path).expect("Failed to open history database");
    let config = NotifyConfig::default();
    let silent = config.general.silent_mode;

    let queue = NotificationQueue::new(config.general.max_queue_size);
    let rules_engine = RulesEngine::new(DefaultRule::default());

    // Start IPC server
    let (ipc_tx, ipc_rx) = mpsc::channel::<Notification>();
    let _ = ipc_server::start_ipc_server(ipc_tx);

    // Attempt Windows notification listener registration
    let _ = listener::request_notification_access();

    let app_state = AppState {
        queue: Mutex::new(queue),
        rules: Mutex::new(rules_engine),
        history: Mutex::new(history_db),
        config: Mutex::new(config),
        silent_mode: Mutex::new(silent),
    };

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            let app_handle = app.handle().clone();
            // Background thread: receive IPC notifications and add to queue
            std::thread::spawn(move || {
                for notif in ipc_rx {
                    let state = app_handle.state::<AppState>();

                    // Evaluate rules
                    let rule_result = state
                        .rules
                        .lock()
                        .unwrap()
                        .evaluate(&notif.source, &notif.priority);

                    if rule_result.action == RuleAction::Block {
                        log::debug!("Blocked notification from {}", notif.source);
                        continue;
                    }

                    // Store in history regardless
                    let _ = state.history.lock().unwrap().store(&notif);

                    // Add to queue
                    state.queue.lock().unwrap().push(notif);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_notifications,
            get_grouped_notifications,
            get_unread_count,
            dismiss_notification,
            dismiss_by_source,
            dismiss_all,
            mark_read,
            send_notification,
            get_rules,
            set_rule,
            remove_rule,
            get_silent_mode,
            set_silent_mode,
            search_history,
            get_recent_history,
            get_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running notify-queue");
}
