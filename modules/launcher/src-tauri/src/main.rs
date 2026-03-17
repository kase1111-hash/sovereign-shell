//! Sovereign Launcher — Main Entry Point
//!
//! A keyboard-driven application launcher replacing the Windows Start Menu.
//! Press Alt+Space to activate, type to search, Enter to launch.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod calc;
mod config;
mod db;
mod icons;
mod indexer;
mod search;

use config::LauncherConfig;
use db::Database;
use icons::IconCache;
use search::AppState;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

/// Register a global hotkey using Win32 API.
/// When the hotkey is pressed, emits a "toggle" event to the frontend.
#[cfg(windows)]
fn register_global_hotkey(app_handle: tauri::AppHandle, config: &LauncherConfig) {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN, MOD_NOREPEAT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetMessageW, MSG, WM_HOTKEY,
    };
    use windows::Win32::Foundation::HWND;

    let modifier = match config.hotkey.modifier.to_lowercase().as_str() {
        "alt" => MOD_ALT,
        "ctrl" | "control" => MOD_CONTROL,
        "shift" => MOD_SHIFT,
        "win" | "super" => MOD_WIN,
        _ => MOD_ALT,
    };

    let vk = match config.hotkey.key.to_lowercase().as_str() {
        "space" => 0x20u32,
        "enter" | "return" => 0x0D,
        "tab" => 0x09,
        key if key.len() == 1 => {
            key.chars().next().unwrap().to_ascii_uppercase() as u32
        }
        _ => 0x20, // default to space
    };

    let hotkey_id = 1i32;

    std::thread::spawn(move || {
        unsafe {
            let result = RegisterHotKey(
                HWND::default(),
                hotkey_id,
                modifier | MOD_NOREPEAT,
                vk,
            );

            if result.is_err() {
                eprintln!("[launcher] Failed to register hotkey — it may be in use by another application");
                return;
            }

            println!("[launcher] Global hotkey registered");

            // Message loop to receive hotkey events
            let mut msg = MSG::default();
            loop {
                let ret = GetMessageW(&mut msg, HWND::default(), 0, 0);
                if ret.0 <= 0 {
                    break;
                }
                if msg.message == WM_HOTKEY && msg.wParam.0 == hotkey_id as usize {
                    // Emit toggle event to the Tauri app
                    let _ = app_handle.emit("toggle-window", ());
                }
            }
        }
    });
}

#[cfg(not(windows))]
fn register_global_hotkey(_app_handle: tauri::AppHandle, _config: &LauncherConfig) {
    eprintln!("[launcher] Global hotkey registration requires Windows");
}

/// Toggle the main window visibility.
#[tauri::command]
fn toggle_window(window: tauri::Window) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
        // Emit event so frontend knows to focus the input
        let _ = window.emit("window-shown", ());
    }
}

/// Hide the window (called on Escape or focus loss).
#[tauri::command]
fn hide_window(window: tauri::Window) {
    let _ = window.hide();
}

fn main() {
    // Load config
    let config: LauncherConfig =
        sovereign_config::load_or_default("launcher").unwrap_or_default();

    let max_results = config.indexing.max_results;
    let extra_dirs = config.indexing.extra_dirs.clone();
    let refresh_seconds = config.indexing.refresh_interval_seconds;

    // Open database
    let db = Database::open_default().expect("Failed to open launcher database");

    // Run initial index
    println!("[launcher] Running initial index...");
    match indexer::run_full_index(&db, &extra_dirs) {
        Ok(stats) => println!(
            "[launcher] Indexed {} apps ({} shortcuts, {} PATH executables, {} extras, {} pruned)",
            stats.total, stats.apps_indexed, stats.path_exes, stats.extras, stats.pruned
        ),
        Err(e) => eprintln!("[launcher] Indexing error: {}", e),
    }

    // Initialize icon cache
    let icon_cache_dir = sovereign_config::data_dir("launcher")
        .map(|d| d.join("icon-cache"))
        .unwrap_or_else(|_| std::env::temp_dir().join("sovereign-launcher-icons"));
    let icon_cache = Arc::new(IconCache::new(icon_cache_dir));

    let app_state = AppState {
        db: Mutex::new(db),
        max_results,
        icon_cache,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            toggle_window,
            hide_window,
            search::search,
            search::record_launch,
            search::launch_app,
            search::open_containing_folder,
            search::get_index_count,
            search::evaluate_calc,
            search::get_icon,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let config_clone = config.clone();

            // Register global hotkey in a background thread
            register_global_hotkey(handle.clone(), &config_clone);

            // Listen for toggle events from the hotkey thread
            let main_window = app.get_webview_window("main")
                .expect("Main window not found");

            let win = main_window.clone();
            app.listen("toggle-window", move |_| {
                if win.is_visible().unwrap_or(false) {
                    let _ = win.hide();
                } else {
                    let _ = win.show();
                    let _ = win.set_focus();
                    let _ = win.emit("window-shown", ());
                }
            });

            // Hide on focus loss
            let win2 = main_window.clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = win2.hide();
                }
            });

            // Apply window vibrancy effect (Windows acrylic blur)
            #[cfg(windows)]
            if config_clone.appearance.enable_vibrancy {
                use window_vibrancy::apply_acrylic;
                let _ = apply_acrylic(&main_window, Some((18, 18, 26, 200)));
            }

            // Periodic re-indexing
            let handle2 = handle.clone();
            let extra = extra_dirs.clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(refresh_seconds));
                    println!("[launcher] Re-indexing...");
                    if let Some(state) = handle2.try_state::<AppState>() {
                        if let Ok(db) = state.db.lock() {
                            match indexer::run_full_index(&db, &extra) {
                                Ok(stats) => println!("[launcher] Re-index: {} total apps", stats.total),
                                Err(e) => eprintln!("[launcher] Re-index error: {}", e),
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Sovereign Launcher");
}
