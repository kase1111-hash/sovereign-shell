//! Directory listing, file metadata, and icon extraction.

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// A single directory entry for the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub is_symlink: bool,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: i64,
    pub created: i64,
    pub readonly: bool,
}

/// Summary of a directory listing.
#[derive(Debug, Clone, Serialize)]
pub struct DirListing {
    pub path: String,
    pub entries: Vec<FileEntry>,
    pub total_items: usize,
    pub total_size: u64,
}

/// Detailed metadata for a single file.
#[derive(Debug, Clone, Serialize)]
pub struct FileDetails {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub created: i64,
    pub accessed: i64,
    pub readonly: bool,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub sha256: Option<String>,
}

/// A drive/volume entry.
#[derive(Debug, Clone, Serialize)]
pub struct DriveInfo {
    pub path: String,
    pub label: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub is_removable: bool,
}

/// List the contents of a directory.
pub fn list_directory(path: &str, show_hidden: bool) -> Result<DirListing, String> {
    let dir_path = PathBuf::from(path);
    if !dir_path.exists() {
        return Err(format!("Directory does not exist: {}", path));
    }
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let read_dir = std::fs::read_dir(&dir_path)
        .map_err(|e| format!("Cannot read directory: {e}"))?;

    let mut entries = Vec::new();
    let mut total_size = 0u64;

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().to_string();
        let is_hidden = is_hidden_file(&name, &entry.path());

        if !show_hidden && is_hidden {
            continue;
        }

        let extension = entry.path()
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase());

        let size = if metadata.is_file() { metadata.len() } else { 0 };
        total_size += size;

        let modified = metadata.modified().ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let created = metadata.created().ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        entries.push(FileEntry {
            name,
            path: entry.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            is_hidden,
            is_symlink: metadata.is_symlink(),
            extension,
            size,
            modified,
            created,
            readonly: metadata.permissions().readonly(),
        });
    }

    let total_items = entries.len();

    Ok(DirListing {
        path: dir_path.to_string_lossy().to_string(),
        entries,
        total_items,
        total_size,
    })
}

/// Get detailed metadata for a single file.
pub fn get_file_details(path: &str) -> Result<FileDetails, String> {
    let file_path = PathBuf::from(path);
    let metadata = std::fs::metadata(&file_path)
        .map_err(|e| format!("Cannot read metadata: {e}"))?;
    let symlink_meta = std::fs::symlink_metadata(&file_path).ok();

    let name = file_path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let ts = |t: std::io::Result<std::time::SystemTime>| -> i64 {
        t.ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    };

    let is_symlink = symlink_meta
        .as_ref()
        .map(|m| m.is_symlink())
        .unwrap_or(false);

    let symlink_target = if is_symlink {
        std::fs::read_link(&file_path)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    } else {
        None
    };

    Ok(FileDetails {
        name,
        path: path.to_string(),
        is_dir: metadata.is_dir(),
        size: metadata.len(),
        modified: ts(metadata.modified()),
        created: ts(metadata.created()),
        accessed: ts(metadata.accessed()),
        readonly: metadata.permissions().readonly(),
        is_symlink,
        symlink_target,
        sha256: None, // Computed on demand via separate command
    })
}

/// Compute SHA-256 hash of a file.
pub fn compute_sha256(path: &str) -> Result<String, String> {
    use sha2::{Sha256, Digest};
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("Cannot open file: {e}"))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)
        .map_err(|e| format!("Hash error: {e}"))?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// List available drives/volumes.
pub fn list_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    #[cfg(windows)]
    {
        // Check drive letters A-Z
        for letter in b'A'..=b'Z' {
            let path = format!("{}:\\", letter as char);
            let p = PathBuf::from(&path);
            if p.exists() {
                drives.push(DriveInfo {
                    path: path.clone(),
                    label: format!("{}:", letter as char),
                    total_bytes: 0,
                    free_bytes: 0,
                    is_removable: false,
                });
            }
        }
    }

    #[cfg(not(windows))]
    {
        // On Linux/Mac, list mount points
        if PathBuf::from("/").exists() {
            drives.push(DriveInfo {
                path: "/".to_string(),
                label: "Root".to_string(),
                total_bytes: 0,
                free_bytes: 0,
                is_removable: false,
            });
        }
        if let Some(home) = std::env::var_os("HOME") {
            drives.push(DriveInfo {
                path: home.to_string_lossy().to_string(),
                label: "Home".to_string(),
                total_bytes: 0,
                free_bytes: 0,
                is_removable: false,
            });
        }
    }

    drives
}

/// Get the children directory names for the sidebar tree (lazy load).
pub fn get_child_dirs(path: &str) -> Result<Vec<String>, String> {
    let dir_path = PathBuf::from(path);
    let read_dir = std::fs::read_dir(&dir_path)
        .map_err(|e| format!("Cannot read directory: {e}"))?;

    let mut dirs = Vec::new();
    for entry in read_dir {
        if let Ok(entry) = entry {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !is_hidden_file(&name, &entry.path()) {
                        dirs.push(name);
                    }
                }
            }
        }
    }
    dirs.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    Ok(dirs)
}

/// Check if a file is hidden (platform-specific).
fn is_hidden_file(name: &str, _path: &Path) -> bool {
    if name.starts_with('.') {
        return true;
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(meta) = std::fs::metadata(_path) {
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            return meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
        }
    }

    false
}
