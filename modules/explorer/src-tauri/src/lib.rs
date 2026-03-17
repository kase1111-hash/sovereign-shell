//! Sovereign Explorer — Tauri library crate.

pub mod archive;
pub mod batch;
pub mod bookmarks;
pub mod config;
pub mod fs_ops;
pub mod fs_read;
pub mod search_client;

use config::ExplorerConfig;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

/// Shared application state.
pub struct AppState {
    pub config: ExplorerConfig,
    pub show_hidden: Mutex<bool>,
    pub clipboard: Mutex<Clipboard>,
}

/// Clipboard for cut/copy operations.
#[derive(Debug, Default)]
pub struct Clipboard {
    pub paths: Vec<String>,
    pub is_cut: bool,
}

/// Result wrapper for Tauri commands.
#[derive(Serialize)]
pub struct CmdResult<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

// ─── Directory Listing Commands ─────────────────────────────────────

#[tauri::command]
pub fn list_directory(path: String, state: State<'_, AppState>) -> Result<fs_read::DirListing, String> {
    let show_hidden = *state.show_hidden.lock().unwrap();
    fs_read::list_directory(&path, show_hidden)
}

#[tauri::command]
pub fn get_file_details(path: String) -> Result<fs_read::FileDetails, String> {
    fs_read::get_file_details(&path)
}

#[tauri::command]
pub fn compute_sha256(path: String) -> Result<String, String> {
    fs_read::compute_sha256(&path)
}

#[tauri::command]
pub fn list_drives() -> Vec<fs_read::DriveInfo> {
    fs_read::list_drives()
}

#[tauri::command]
pub fn get_child_dirs(path: String) -> Result<Vec<String>, String> {
    fs_read::get_child_dirs(&path)
}

#[tauri::command]
pub fn get_bookmarks() -> Vec<bookmarks::Bookmark> {
    bookmarks::default_bookmarks()
}

// ─── File Operation Commands ────────────────────────────────────────

#[tauri::command]
pub fn copy_items(sources: Vec<String>, dest_dir: String) -> Result<usize, String> {
    fs_ops::copy_items(&sources, &dest_dir)
}

#[tauri::command]
pub fn move_items(sources: Vec<String>, dest_dir: String) -> Result<usize, String> {
    fs_ops::move_items(&sources, &dest_dir)
}

#[tauri::command]
pub fn delete_to_trash(paths: Vec<String>) -> Result<usize, String> {
    fs_ops::delete_to_trash(&paths)
}

#[tauri::command]
pub fn delete_permanent(paths: Vec<String>) -> Result<usize, String> {
    fs_ops::delete_permanent(&paths)
}

#[tauri::command]
pub fn rename_item(path: String, new_name: String) -> Result<String, String> {
    fs_ops::rename_item(&path, &new_name)
}

#[tauri::command]
pub fn create_directory(parent: String, name: String) -> Result<String, String> {
    fs_ops::create_directory(&parent, &name)
}

#[tauri::command]
pub fn create_file(parent: String, name: String) -> Result<String, String> {
    fs_ops::create_file(&parent, &name)
}

// ─── Clipboard Commands ─────────────────────────────────────────────

#[tauri::command]
pub fn clipboard_copy(paths: Vec<String>, state: State<'_, AppState>) {
    let mut clip = state.clipboard.lock().unwrap();
    clip.paths = paths;
    clip.is_cut = false;
}

#[tauri::command]
pub fn clipboard_cut(paths: Vec<String>, state: State<'_, AppState>) {
    let mut clip = state.clipboard.lock().unwrap();
    clip.paths = paths;
    clip.is_cut = true;
}

#[tauri::command]
pub fn clipboard_paste(dest_dir: String, state: State<'_, AppState>) -> Result<usize, String> {
    let mut clip = state.clipboard.lock().unwrap();
    if clip.paths.is_empty() {
        return Ok(0);
    }

    let result = if clip.is_cut {
        let r = fs_ops::move_items(&clip.paths, &dest_dir);
        if r.is_ok() {
            clip.paths.clear();
            clip.is_cut = false;
        }
        r
    } else {
        fs_ops::copy_items(&clip.paths, &dest_dir)
    };

    result
}

#[tauri::command]
pub fn clipboard_has_items(state: State<'_, AppState>) -> bool {
    !state.clipboard.lock().unwrap().paths.is_empty()
}

// ─── Settings Commands ──────────────────────────────────────────────

#[tauri::command]
pub fn toggle_hidden(state: State<'_, AppState>) -> bool {
    let mut show = state.show_hidden.lock().unwrap();
    *show = !*show;
    *show
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> ExplorerConfig {
    state.config.clone()
}

// ─── Search Commands ────────────────────────────────────────────────

#[tauri::command]
pub fn search_files(
    query: String,
    max_results: Option<usize>,
    file_types: Option<Vec<String>>,
) -> Result<Vec<search_client::SearchResult>, String> {
    search_client::search(
        &query,
        max_results.unwrap_or(20),
        &file_types.unwrap_or_default(),
    )
}

#[tauri::command]
pub fn search_daemon_available() -> bool {
    search_client::is_available()
}

// ─── Archive Commands ───────────────────────────────────────────────

#[tauri::command]
pub fn list_archive(path: String) -> Result<Vec<archive::ArchiveEntry>, String> {
    archive::list_zip(&path)
}

#[tauri::command]
pub fn extract_archive(archive_path: String, dest_dir: String) -> Result<usize, String> {
    archive::extract_zip(&archive_path, &dest_dir)
}

#[tauri::command]
pub fn create_archive(sources: Vec<String>, archive_path: String) -> Result<usize, String> {
    archive::create_zip(&sources, &archive_path)
}

// ─── Batch Rename Commands ──────────────────────────────────────────

#[tauri::command]
pub fn preview_batch_rename(
    paths: Vec<String>,
    pattern: batch::RenamePattern,
) -> Vec<batch::RenamePreview> {
    batch::preview_batch_rename(&paths, &pattern)
}

#[tauri::command]
pub fn execute_batch_rename(previews: Vec<batch::RenamePreview>) -> Result<usize, String> {
    batch::execute_batch_rename(&previews)
}

#[tauri::command]
pub fn preview_regex_rename(
    paths: Vec<String>,
    find_pattern: String,
    replace_with: String,
) -> Result<Vec<batch::RenamePreview>, String> {
    batch::preview_regex_rename(&paths, &find_pattern, &replace_with)
}

// ─── Open With System Default ───────────────────────────────────────

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| format!("Open failed: {e}"))
}

#[tauri::command]
pub fn open_in_terminal(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let shell = &state.config.general.default_shell;
    let dir = if std::path::Path::new(&path).is_dir() {
        path
    } else {
        std::path::Path::new(&path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_string_lossy()
            .to_string()
    };

    std::process::Command::new(shell)
        .current_dir(&dir)
        .spawn()
        .map_err(|e| format!("Failed to open terminal: {e}"))?;
    Ok(())
}

// ─── Home Directory ─────────────────────────────────────────────────

#[tauri::command]
pub fn get_home_dir() -> String {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string())
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
    }
}

// ─── Read file content for preview ──────────────────────────────────

#[tauri::command]
pub fn read_text_preview(path: String, max_lines: Option<usize>) -> Result<String, String> {
    use std::io::{BufRead, BufReader};
    let max = max_lines.unwrap_or(200);
    let file = std::fs::File::open(&path)
        .map_err(|e| format!("Cannot open file: {e}"))?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines()
        .take(max)
        .filter_map(|l| l.ok())
        .collect();
    Ok(lines.join("\n"))
}

// ─── App Builder ────────────────────────────────────────────────────

pub fn run() {
    let config: ExplorerConfig =
        sovereign_config::load_or_default("explorer").unwrap_or_default();

    let show_hidden = config.general.show_hidden_files;

    let state = AppState {
        config,
        show_hidden: Mutex::new(show_hidden),
        clipboard: Mutex::new(Clipboard::default()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // Directory listing
            list_directory,
            get_file_details,
            compute_sha256,
            list_drives,
            get_child_dirs,
            get_bookmarks,
            get_home_dir,
            // File operations
            copy_items,
            move_items,
            delete_to_trash,
            delete_permanent,
            rename_item,
            create_directory,
            create_file,
            // Clipboard
            clipboard_copy,
            clipboard_cut,
            clipboard_paste,
            clipboard_has_items,
            // Settings
            toggle_hidden,
            get_config,
            // Search
            search_files,
            search_daemon_available,
            // Archives
            list_archive,
            extract_archive,
            create_archive,
            // Batch rename
            preview_batch_rename,
            execute_batch_rename,
            preview_regex_rename,
            // Open
            open_file,
            open_in_terminal,
            // Preview
            read_text_preview,
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
        .expect("error while running Sovereign Explorer");
}
