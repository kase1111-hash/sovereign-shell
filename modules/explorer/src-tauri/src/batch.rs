//! Batch rename with pattern support and preview.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A rename preview entry showing before and after names.
#[derive(Debug, Clone, Serialize)]
pub struct RenamePreview {
    pub original_path: String,
    pub original_name: String,
    pub new_name: String,
    pub error: Option<String>,
}

/// Batch rename patterns.
#[derive(Debug, Clone, Deserialize)]
pub struct RenamePattern {
    /// Pattern with placeholders: {name}, {ext}, {counter}, {date}
    pub pattern: String,
    /// Starting counter value.
    #[serde(default = "default_counter_start")]
    pub counter_start: usize,
    /// Counter zero-padding width.
    #[serde(default = "default_counter_width")]
    pub counter_width: usize,
}

fn default_counter_start() -> usize { 1 }
fn default_counter_width() -> usize { 3 }

/// Generate a preview of what batch rename would produce.
pub fn preview_batch_rename(paths: &[String], pattern: &RenamePattern) -> Vec<RenamePreview> {
    paths.iter().enumerate().map(|(i, path)| {
        let p = PathBuf::from(path);
        let original_name = p.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let stem = p.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let ext = p.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        let counter = pattern.counter_start + i;
        let counter_str = format!("{:0>width$}", counter, width = pattern.counter_width);

        let date = chrono::Local::now().format("%Y-%m-%d").to_string();

        let new_name = pattern.pattern
            .replace("{name}", &stem)
            .replace("{ext}", &ext)
            .replace("{counter}", &counter_str)
            .replace("{date}", &date);

        // Add extension back if not in pattern
        let new_name = if !pattern.pattern.contains("{ext}") && !ext.is_empty() {
            format!("{}.{}", new_name, ext)
        } else {
            new_name
        };

        let error = if new_name.is_empty() {
            Some("Empty name".to_string())
        } else if new_name.contains('/') || new_name.contains('\\') {
            Some("Name contains path separator".to_string())
        } else {
            None
        };

        RenamePreview {
            original_path: path.clone(),
            original_name,
            new_name,
            error,
        }
    }).collect()
}

/// Execute a batch rename from previewed results.
pub fn execute_batch_rename(previews: &[RenamePreview]) -> Result<usize, String> {
    let mut count = 0;

    for preview in previews {
        if preview.error.is_some() {
            continue;
        }

        let src = PathBuf::from(&preview.original_path);
        let parent = src.parent()
            .ok_or_else(|| format!("No parent for {}", preview.original_path))?;
        let target = parent.join(&preview.new_name);

        if target.exists() {
            return Err(format!(
                "Cannot rename '{}' to '{}': target already exists",
                preview.original_name, preview.new_name
            ));
        }

        std::fs::rename(&src, &target)
            .map_err(|e| format!("Rename failed for '{}': {e}", preview.original_name))?;
        count += 1;
    }

    Ok(count)
}

/// Regex-based batch rename preview.
pub fn preview_regex_rename(
    paths: &[String],
    find_pattern: &str,
    replace_with: &str,
) -> Result<Vec<RenamePreview>, String> {
    let re = regex_lite::Regex::new(find_pattern)
        .map_err(|e| format!("Invalid regex: {e}"))?;

    let previews = paths.iter().map(|path| {
        let p = PathBuf::from(path);
        let original_name = p.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let new_name = re.replace_all(&original_name, replace_with).to_string();

        RenamePreview {
            original_path: path.clone(),
            original_name,
            new_name,
            error: None,
        }
    }).collect();

    Ok(previews)
}
