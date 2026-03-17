//! Sovereign Shell — Shared Configuration Library
//!
//! Provides per-module TOML config loading and saving.
//! Config files live at `%APPDATA%\SovereignShell\<module-name>\config.toml`.
//! If no config exists, a default is written and returned.

use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;

/// Errors that can occur during config operations.
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
    NoAppData,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Parse(e) => write!(f, "Config parse error: {e}"),
            Self::Serialize(e) => write!(f, "Config serialize error: {e}"),
            Self::NoAppData => write!(f, "Could not determine APPDATA directory"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}
impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self { Self::Parse(e) }
}
impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self { Self::Serialize(e) }
}

/// Returns the config directory for a module: `%APPDATA%\SovereignShell\<module_name>\`
pub fn config_dir(module_name: &str) -> Result<PathBuf, ConfigError> {
    let base = dirs::config_dir().ok_or(ConfigError::NoAppData)?;
    Ok(base.join("SovereignShell").join(module_name))
}

/// Returns the full path to a module's config file.
pub fn config_path(module_name: &str) -> Result<PathBuf, ConfigError> {
    Ok(config_dir(module_name)?.join("config.toml"))
}

/// Load config from disk. If the file doesn't exist, write the default and return it.
///
/// The config type must implement `Default`, `Serialize`, and `DeserializeOwned`.
///
/// # Example
/// ```ignore
/// #[derive(Default, Serialize, Deserialize)]
/// struct LauncherConfig {
///     hotkey_modifier: String,
///     hotkey_key: String,
///     max_results: usize,
/// }
///
/// let config: LauncherConfig = sovereign_config::load_or_default("launcher")?;
/// ```
pub fn load_or_default<T>(module_name: &str) -> Result<T, ConfigError>
where
    T: Default + Serialize + DeserializeOwned,
{
    let path = config_path(module_name)?;

    if path.exists() {
        let contents = fs::read_to_string(&path)?;
        let config: T = toml::from_str(&contents)?;
        Ok(config)
    } else {
        let default = T::default();
        save(module_name, &default)?;
        Ok(default)
    }
}

/// Load config from disk. Returns None if the file doesn't exist.
pub fn load<T>(module_name: &str) -> Result<Option<T>, ConfigError>
where
    T: DeserializeOwned,
{
    let path = config_path(module_name)?;

    if path.exists() {
        let contents = fs::read_to_string(&path)?;
        let config: T = toml::from_str(&contents)?;
        Ok(Some(config))
    } else {
        Ok(None)
    }
}

/// Save config to disk. Creates the directory if it doesn't exist.
pub fn save<T: Serialize>(module_name: &str, config: &T) -> Result<(), ConfigError> {
    let path = config_path(module_name)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = toml::to_string_pretty(config)?;
    fs::write(&path, contents)?;
    Ok(())
}

/// Returns the data directory for a module (for databases, caches, etc.)
/// `%APPDATA%\SovereignShell\<module_name>\data\`
pub fn data_dir(module_name: &str) -> Result<PathBuf, ConfigError> {
    let dir = config_dir(module_name)?.join("data");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns the log directory for a module.
/// `%APPDATA%\SovereignShell\<module_name>\logs\`
pub fn log_dir(module_name: &str) -> Result<PathBuf, ConfigError> {
    let dir = config_dir(module_name)?.join("logs");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
