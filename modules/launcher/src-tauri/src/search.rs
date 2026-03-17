//! Search commands exposed to the Tauri frontend.
//!
//! These are the `#[tauri::command]` functions that the JavaScript frontend calls
//! via `invoke()`.

use crate::db::{Database, SearchResult};
use crate::icons::IconCache;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Shared application state.
pub struct AppState {
    pub db: Mutex<Database>,
    pub max_results: usize,
    pub icon_cache: Arc<IconCache>,
}

/// Search for applications matching the query.
#[tauri::command]
pub fn search(query: &str, state: State<'_, AppState>) -> Result<Vec<SearchResult>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {e}"))?;
    db.search(query, state.max_results)
        .map_err(|e| format!("Search error: {e}"))
}

/// Record that an app was launched (updates ranking).
#[tauri::command]
pub fn record_launch(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {e}"))?;
    db.record_launch(id).map_err(|e| format!("DB error: {e}"))
}

/// Launch an application by path.
/// Uses ShellExecuteW on Windows for proper .lnk handling.
#[tauri::command]
pub fn launch_app(path: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::ffi::OsStr;
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let wide_path: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();
        let wide_open: Vec<u16> = OsStr::new("open").encode_wide().chain(Some(0)).collect();

        unsafe {
            ShellExecuteW(
                None,
                PCWSTR(wide_open.as_ptr()),
                PCWSTR(wide_path.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
        }
        Ok(())
    }

    #[cfg(not(windows))]
    {
        // Fallback for dev on Linux
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Launch error: {e}"))?;
        Ok(())
    }
}

/// Open the folder containing a file.
#[tauri::command]
pub fn open_containing_folder(path: &str) -> Result<(), String> {
    let parent = std::path::Path::new(path)
        .parent()
        .ok_or_else(|| "No parent directory".to_string())?;

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(parent.as_os_str())
            .spawn()
            .map_err(|e| format!("Open folder error: {e}"))?;
    }

    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open")
            .arg(parent.as_os_str())
            .spawn()
            .map_err(|e| format!("Open folder error: {e}"))?;
    }

    Ok(())
}

/// Get the number of indexed applications.
#[tauri::command]
pub fn get_index_count(state: State<'_, AppState>) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {e}"))?;
    db.count().map_err(|e| format!("DB error: {e}"))
}

/// Evaluate a math expression (calculator mode).
/// Returns a JSON object with `result` (formatted string) and `value` (f64).
#[tauri::command]
pub fn evaluate_calc(expr: &str) -> Result<CalcResult, String> {
    let value = crate::calc::evaluate(expr)?;
    Ok(CalcResult {
        display: crate::calc::format_result(value),
        value,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CalcResult {
    pub display: String,
    pub value: f64,
}

/// Get the cached icon path for an executable.
/// Returns the local file path to a PNG icon, or null if unavailable.
#[tauri::command]
pub fn get_icon(exe_path: &str, state: State<'_, AppState>) -> Option<String> {
    state.icon_cache.get_or_extract(exe_path)
}
