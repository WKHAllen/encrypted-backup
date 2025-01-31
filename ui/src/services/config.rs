//! Application state configuration.

use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::env::current_exe;
use std::io;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Returns the path to the configuration file.
fn config_file_path() -> io::Result<PathBuf> {
    let current = current_exe()?;
    let here = current.parent().unwrap_or_else(|| Path::new("."));
    let joined = Path::new(here).join(CONFIG_FILE_NAME);
    Ok(joined)
}

/// The application state backup configuration with all fields optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct BackupConfigOpt {
    /// The include paths.
    include_paths: Option<Vec<PathBuf>>,
    /// The output path.
    output_path: Option<PathBuf>,
    /// The exclusion globs.
    exclude_globs: Option<Vec<String>>,
    /// The chunk size magnitude.
    chunk_size_magnitude: Option<u8>,
    /// The task pool size.
    pool_size: Option<u8>,
    /// Whether the basic config options are open.
    basic_config_open: Option<bool>,
    /// Whether the advanced config options are open.
    advanced_config_open: Option<bool>,
}

/// The application state backup configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackupConfig {
    /// The include paths.
    pub include_paths: Vec<PathBuf>,
    /// The output path.
    pub output_path: Option<PathBuf>,
    /// The exclusion globs.
    pub exclude_globs: Vec<String>,
    /// The chunk size magnitude.
    pub chunk_size_magnitude: u8,
    /// The task pool size.
    pub pool_size: u8,
    /// Whether the basic config options are open.
    pub basic_config_open: bool,
    /// Whether the advanced config options are open.
    pub advanced_config_open: bool,
}

impl From<BackupConfigOpt> for BackupConfig {
    fn from(value: BackupConfigOpt) -> Self {
        Self {
            include_paths: value.include_paths.unwrap_or_default(),
            output_path: value.output_path,
            exclude_globs: value.exclude_globs.unwrap_or_default(),
            chunk_size_magnitude: value.chunk_size_magnitude.unwrap_or(16),
            pool_size: value.pool_size.unwrap_or(4),
            basic_config_open: value.basic_config_open.unwrap_or(true),
            advanced_config_open: value.advanced_config_open.unwrap_or(false),
        }
    }
}

impl From<BackupConfig> for BackupConfigOpt {
    fn from(value: BackupConfig) -> Self {
        Self {
            include_paths: Some(value.include_paths),
            output_path: value.output_path,
            exclude_globs: Some(value.exclude_globs),
            chunk_size_magnitude: Some(value.chunk_size_magnitude),
            pool_size: Some(value.pool_size),
            basic_config_open: Some(value.basic_config_open),
            advanced_config_open: Some(value.advanced_config_open),
        }
    }
}

/// The application state extraction configuration with all fields optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct ExtractionConfigOpt {
    /// The backup path.
    backup_path: Option<PathBuf>,
    /// The output path.
    output_path: Option<PathBuf>,
    /// The pool size.
    pool_size: Option<u8>,
    /// Whether the basic config options are open.
    basic_config_open: Option<bool>,
    /// Whether the advanced config options are open.
    advanced_config_open: Option<bool>,
}

/// The application state extraction configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtractionConfig {
    /// The backup path.
    pub backup_path: Option<PathBuf>,
    /// The output path.
    pub output_path: Option<PathBuf>,
    /// The pool size.
    pub pool_size: u8,
    /// Whether the basic config options are open.
    pub basic_config_open: bool,
    /// Whether the advanced config options are open.
    pub advanced_config_open: bool,
}

impl From<ExtractionConfigOpt> for ExtractionConfig {
    fn from(value: ExtractionConfigOpt) -> Self {
        Self {
            backup_path: value.backup_path,
            output_path: value.output_path,
            pool_size: value.pool_size.unwrap_or(4),
            basic_config_open: value.basic_config_open.unwrap_or(true),
            advanced_config_open: value.advanced_config_open.unwrap_or(false),
        }
    }
}

impl From<ExtractionConfig> for ExtractionConfigOpt {
    fn from(value: ExtractionConfig) -> Self {
        Self {
            backup_path: value.backup_path,
            output_path: value.output_path,
            pool_size: Some(value.pool_size),
            basic_config_open: Some(value.basic_config_open),
            advanced_config_open: Some(value.advanced_config_open),
        }
    }
}

/// The application state configuration with all fields optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct ConfigOpt {
    /// The backup configuration options.
    backup_config: BackupConfigOpt,
    /// The extraction configuration options.
    extraction_config: ExtractionConfigOpt,
}

/// The application state configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    /// The backup configuration options.
    pub backup_config: BackupConfig,
    /// The extraction configuration options.
    pub extraction_config: ExtractionConfig,
}

impl From<ConfigOpt> for Config {
    fn from(value: ConfigOpt) -> Self {
        Self {
            backup_config: value.backup_config.into(),
            extraction_config: value.extraction_config.into(),
        }
    }
}

impl From<Config> for ConfigOpt {
    fn from(value: Config) -> Self {
        Self {
            backup_config: value.backup_config.into(),
            extraction_config: value.extraction_config.into(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from(ConfigOpt::default())
    }
}

impl Config {
    /// Loads the configuration state from the file.
    pub async fn load() -> io::Result<Self> {
        let config_path = config_file_path()?;

        if config_path.exists() {
            let config_bytes = fs::read(config_path).await?;
            let config_opt = serde_json::from_slice::<ConfigOpt>(&config_bytes)?;
            Ok(Self::from(config_opt))
        } else {
            Ok(Self::default())
        }
    }

    /// Saves the configuration state to the file.
    pub async fn save(&self) -> io::Result<()> {
        let config_path = config_file_path()?;
        let config_opt = ConfigOpt::from(self.clone());
        let config_bytes = serde_json::to_vec(&config_opt)?;
        fs::write(config_path, config_bytes).await?;
        Ok(())
    }
}
