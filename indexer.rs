//! Application indexer for the Sovereign Launcher.
//!
//! Scans the Start Menu directories and PATH for launchable applications,
//! extracts names and keywords, and populates the SQLite database.

use crate::db::Database;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Directories to scan for Start Menu shortcuts.
fn start_menu_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User Start Menu
    if let Some(appdata) = std::env::var_os("APPDATA") {
        dirs.push(
            PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs"),
        );
    }

    // System-wide Start Menu
    if let Some(progdata) = std::env::var_os("ProgramData") {
        dirs.push(
            PathBuf::from(progdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs"),
        );
    }

    dirs
}

/// Extract the display name from a file path.
/// For .lnk files, strips the extension. For executables, strips extension and
/// replaces hyphens/underscores with spaces.
fn display_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Clean up common patterns
    stem.replace('_', " ").replace('-', " ")
}

/// Generate search keywords from a path and name.
/// Includes the name tokens, parent folder name, and file extension.
fn generate_keywords(name: &str, path: &Path) -> String {
    let mut keywords = Vec::new();

    // Name tokens
    for word in name.split_whitespace() {
        keywords.push(word.to_lowercase());
    }

    // Parent directory name (often the "publisher" or "category")
    if let Some(parent) = path.parent() {
        if let Some(parent_name) = parent.file_name() {
            let pname = parent_name.to_string_lossy().to_lowercase();
            if pname != "programs" && pname != "start menu" {
                keywords.push(pname);
            }
        }
    }

    keywords.dedup();
    keywords.join(" ")
}

/// Recursively find all .lnk files in a directory.
fn find_shortcuts(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }

    let walker = walkdir_recursive(dir);
    for entry in walker {
        let ext = entry
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if ext == "lnk" {
            results.push(entry);
        }
    }
    results
}

/// Simple recursive directory walk (avoids external walkdir dependency).
fn walkdir_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir_recursive(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

/// Find executables on PATH.
fn find_path_executables() -> Vec<PathBuf> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    let path_var = std::env::var("PATH").unwrap_or_default();
    let exe_extensions = ["exe", "cmd", "bat", "ps1", "com"];

    for dir in std::env::split_paths(&path_var) {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let ext = path
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();
                if exe_extensions.contains(&ext.as_str()) {
                    let name_lower = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase();
                    if seen.insert(name_lower) {
                        results.push(path);
                    }
                }
            }
        }
    }
    results
}

/// Resolve a .lnk shortcut to its target path.
/// On Windows, uses COM IShellLink. On other platforms, returns the .lnk path itself.
#[cfg(windows)]
fn resolve_shortcut(lnk_path: &Path) -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::core::{Interface, HSTRING};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink, IPersistFile};

    unsafe {
        // COM init (safe to call multiple times)
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let shell_link: IShellLinkW =
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist_file: IPersistFile = shell_link.cast().ok()?;

        let wide_path = HSTRING::from(lnk_path.as_os_str());
        persist_file.Load(&wide_path, 0).ok()?;

        let mut target_buf = [0u16; 260];
        shell_link
            .GetPath(&mut target_buf, std::ptr::null_mut(), 0)
            .ok()?;

        let len = target_buf.iter().position(|&c| c == 0).unwrap_or(target_buf.len());
        if len == 0 {
            return None;
        }

        let target = OsString::from_wide(&target_buf[..len]);
        let target_path = PathBuf::from(target);

        if target_path.exists() {
            Some(target_path)
        } else {
            // Shortcut target doesn't exist — index the .lnk itself
            Some(lnk_path.to_path_buf())
        }
    }
}

#[cfg(not(windows))]
fn resolve_shortcut(lnk_path: &Path) -> Option<PathBuf> {
    // On non-Windows, just return the path as-is (for development)
    Some(lnk_path.to_path_buf())
}

/// Run a full index scan. Populates the database with all discoverable applications.
pub fn run_full_index(db: &Database, extra_dirs: &[String]) -> Result<IndexStats, String> {
    let mut stats = IndexStats::default();

    // Scan Start Menu shortcuts
    let start_dirs = start_menu_dirs();
    for dir in &start_dirs {
        let shortcuts = find_shortcuts(dir);
        for lnk_path in shortcuts {
            let name = display_name(&lnk_path);
            let keywords = generate_keywords(&name, &lnk_path);

            // Use the .lnk path as the launch path (ShellExecute handles .lnk natively)
            let launch_path = lnk_path.to_string_lossy().to_string();

            // Try to resolve target for icon extraction later
            let icon_path = resolve_shortcut(&lnk_path)
                .map(|p| p.to_string_lossy().to_string());

            if let Err(e) = db.upsert_app(&name, &launch_path, &keywords, icon_path.as_deref()) {
                eprintln!("[indexer] Failed to index {}: {}", launch_path, e);
                stats.errors += 1;
            } else {
                stats.apps_indexed += 1;
            }
        }
    }

    // Scan PATH executables
    let path_exes = find_path_executables();
    for exe_path in path_exes {
        let name = display_name(&exe_path);
        let keywords = generate_keywords(&name, &exe_path);
        let path_str = exe_path.to_string_lossy().to_string();

        if let Err(e) = db.upsert_app(&name, &path_str, &keywords, Some(&path_str)) {
            eprintln!("[indexer] Failed to index {}: {}", path_str, e);
            stats.errors += 1;
        } else {
            stats.path_exes += 1;
        }
    }

    // Scan extra directories
    for extra in extra_dirs {
        let dir = PathBuf::from(extra);
        if !dir.exists() {
            continue;
        }
        let shortcuts = find_shortcuts(&dir);
        for lnk_path in shortcuts {
            let name = display_name(&lnk_path);
            let keywords = generate_keywords(&name, &lnk_path);
            let launch_path = lnk_path.to_string_lossy().to_string();

            if let Err(e) = db.upsert_app(&name, &launch_path, &keywords, None) {
                stats.errors += 1;
                eprintln!("[indexer] Failed to index {}: {}", launch_path, e);
            } else {
                stats.extras += 1;
            }
        }
    }

    // Prune entries for apps that no longer exist
    match db.prune_missing() {
        Ok(n) => stats.pruned = n,
        Err(e) => eprintln!("[indexer] Prune failed: {}", e),
    }

    stats.total = db.count().unwrap_or(0);
    Ok(stats)
}

/// Statistics from an indexing run.
#[derive(Debug, Default, serde::Serialize)]
pub struct IndexStats {
    pub apps_indexed: usize,
    pub path_exes: usize,
    pub extras: usize,
    pub pruned: usize,
    pub errors: usize,
    pub total: i64,
}
