//! File operations: copy, move, delete, rename, create.

use log::{info, warn};
use serde::Serialize;
use std::path::PathBuf;

/// Progress update for long-running operations.
#[derive(Debug, Clone, Serialize)]
pub struct OpProgress {
    pub files_done: usize,
    pub files_total: usize,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub current_file: String,
}

/// Copy files/directories to a destination directory.
pub fn copy_items(sources: &[String], dest_dir: &str) -> Result<usize, String> {
    let dest = PathBuf::from(dest_dir);
    if !dest.is_dir() {
        return Err(format!("Destination is not a directory: {}", dest_dir));
    }

    let mut count = 0;
    for source in sources {
        let src = PathBuf::from(source);
        let name = src.file_name()
            .ok_or_else(|| format!("Invalid source path: {}", source))?;
        let target = dest.join(name);

        // Handle name collision
        let target = unique_path(&target);

        if src.is_dir() {
            copy_dir_recursive(&src, &target)?;
        } else {
            std::fs::copy(&src, &target)
                .map_err(|e| format!("Copy failed {}: {e}", source))?;
        }
        count += 1;
    }

    info!("Copied {} items to {}", count, dest_dir);
    Ok(count)
}

/// Move files/directories to a destination directory.
pub fn move_items(sources: &[String], dest_dir: &str) -> Result<usize, String> {
    let dest = PathBuf::from(dest_dir);
    if !dest.is_dir() {
        return Err(format!("Destination is not a directory: {}", dest_dir));
    }

    let mut count = 0;
    for source in sources {
        let src = PathBuf::from(source);
        let name = src.file_name()
            .ok_or_else(|| format!("Invalid source path: {}", source))?;
        let target = dest.join(name);
        let target = unique_path(&target);

        // Try rename first (fastest, same-volume)
        match std::fs::rename(&src, &target) {
            Ok(()) => {}
            Err(_) => {
                // Cross-volume: copy then delete
                if src.is_dir() {
                    copy_dir_recursive(&src, &target)?;
                    std::fs::remove_dir_all(&src)
                        .map_err(|e| format!("Remove after move failed {}: {e}", source))?;
                } else {
                    std::fs::copy(&src, &target)
                        .map_err(|e| format!("Copy for move failed {}: {e}", source))?;
                    std::fs::remove_file(&src)
                        .map_err(|e| format!("Remove after move failed {}: {e}", source))?;
                }
            }
        }
        count += 1;
    }

    info!("Moved {} items to {}", count, dest_dir);
    Ok(count)
}

/// Delete files/directories to the recycle bin.
pub fn delete_to_trash(paths: &[String]) -> Result<usize, String> {
    let mut count = 0;
    for path in paths {
        trash::delete(path)
            .map_err(|e| format!("Trash failed {}: {e}", path))?;
        count += 1;
    }
    info!("Trashed {} items", count);
    Ok(count)
}

/// Permanently delete files/directories.
pub fn delete_permanent(paths: &[String]) -> Result<usize, String> {
    let mut count = 0;
    for path in paths {
        let p = PathBuf::from(path);
        if p.is_dir() {
            std::fs::remove_dir_all(&p)
                .map_err(|e| format!("Delete failed {}: {e}", path))?;
        } else {
            std::fs::remove_file(&p)
                .map_err(|e| format!("Delete failed {}: {e}", path))?;
        }
        count += 1;
    }
    info!("Permanently deleted {} items", count);
    Ok(count)
}

/// Rename a file or directory.
pub fn rename_item(path: &str, new_name: &str) -> Result<String, String> {
    if new_name.is_empty() || new_name.contains('/') || new_name.contains('\\') {
        return Err("Invalid name".to_string());
    }

    let src = PathBuf::from(path);
    let parent = src.parent()
        .ok_or_else(|| "Cannot rename root".to_string())?;
    let target = parent.join(new_name);

    if target.exists() {
        return Err(format!("A file named '{}' already exists", new_name));
    }

    std::fs::rename(&src, &target)
        .map_err(|e| format!("Rename failed: {e}"))?;

    let new_path = target.to_string_lossy().to_string();
    info!("Renamed {} -> {}", path, new_path);
    Ok(new_path)
}

/// Create a new directory.
pub fn create_directory(parent: &str, name: &str) -> Result<String, String> {
    let dir_path = PathBuf::from(parent).join(name);
    if dir_path.exists() {
        return Err(format!("'{}' already exists", name));
    }
    std::fs::create_dir(&dir_path)
        .map_err(|e| format!("Create directory failed: {e}"))?;
    let path = dir_path.to_string_lossy().to_string();
    info!("Created directory: {}", path);
    Ok(path)
}

/// Create a new empty file.
pub fn create_file(parent: &str, name: &str) -> Result<String, String> {
    let file_path = PathBuf::from(parent).join(name);
    if file_path.exists() {
        return Err(format!("'{}' already exists", name));
    }
    std::fs::File::create(&file_path)
        .map_err(|e| format!("Create file failed: {e}"))?;
    let path = file_path.to_string_lossy().to_string();
    info!("Created file: {}", path);
    Ok(path)
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(dst)
        .map_err(|e| format!("Create dir failed: {e}"))?;

    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("Read dir failed: {e}"))?
    {
        let entry = entry.map_err(|e| format!("Dir entry error: {e}"))?;
        let target = dst.join(entry.file_name());

        if entry.file_type().map_err(|e| format!("{e}"))?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)
                .map_err(|e| format!("Copy failed: {e}"))?;
        }
    }
    Ok(())
}

/// Generate a unique path by appending (1), (2), etc. if needed.
fn unique_path(path: &PathBuf) -> PathBuf {
    if !path.exists() {
        return path.clone();
    }

    let stem = path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let ext = path.extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let parent = path.parent().unwrap_or(path);

    for i in 1..1000 {
        let candidate = parent.join(format!("{} ({}){}", stem, i, ext));
        if !candidate.exists() {
            return candidate;
        }
    }
    path.clone()
}
