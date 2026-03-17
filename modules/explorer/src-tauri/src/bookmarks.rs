//! Quick-access bookmarks / favorites.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A bookmarked location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub path: String,
}

/// Load bookmarks from the explorer config.
pub fn default_bookmarks() -> Vec<Bookmark> {
    let mut bookmarks = Vec::new();

    #[cfg(windows)]
    {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let dirs = [
            ("Desktop", "Desktop"),
            ("Documents", "Documents"),
            ("Downloads", "Downloads"),
            ("Pictures", "Pictures"),
            ("Music", "Music"),
            ("Videos", "Videos"),
        ];
        for (name, folder) in &dirs {
            let path = PathBuf::from(&user_profile).join(folder);
            if path.exists() {
                bookmarks.push(Bookmark {
                    name: name.to_string(),
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }

    #[cfg(not(windows))]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let dirs = [
            ("Desktop", "Desktop"),
            ("Documents", "Documents"),
            ("Downloads", "Downloads"),
            ("Pictures", "Pictures"),
        ];
        for (name, folder) in &dirs {
            let path = PathBuf::from(&home).join(folder);
            if path.exists() {
                bookmarks.push(Bookmark {
                    name: name.to_string(),
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }

    bookmarks
}
