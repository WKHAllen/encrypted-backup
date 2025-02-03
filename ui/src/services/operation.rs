//! API to trigger a backup or extraction operation.

use backup::BackupResult;
use glob::Pattern;
use std::path::PathBuf;

/// Perform the configured backup operation.
#[allow(clippy::needless_pass_by_value)]
fn perform_backup(
    include_paths: Vec<PathBuf>,
    output_path: PathBuf,
    exclude_globs: Vec<Pattern>,
    chunk_size_magnitude: u8,
    pool_size: u8,
    password: String,
) -> BackupResult<PathBuf> {
    backup::backup(
        &include_paths,
        &exclude_globs,
        output_path,
        &password,
        1 << chunk_size_magnitude,
        pool_size,
    )
}

/// Perform the configured extraction operation.
#[allow(clippy::needless_pass_by_value)]
fn perform_extraction(
    backup_path: PathBuf,
    output_path: PathBuf,
    pool_size: u8,
    password: String,
) -> BackupResult<PathBuf> {
    backup::extract(backup_path, output_path, &password, pool_size)
}

/// A representation of a backup or extraction operation.
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    /// A backup operation.
    Backup {
        /// The list of paths to include in the backup.
        include_paths: Vec<PathBuf>,
        /// The path in which to place the completed backup.
        output_path: PathBuf,
        /// The list of glob patterns to exclude from the backup.
        exclude_globs: Vec<Pattern>,
        /// The magnitude of the backup's chunk sizes.
        chunk_size_magnitude: u8,
        /// The number of threads to spawn in the thread pool.
        pool_size: u8,
        /// The backup password.
        password: String,
    },
    /// An extraction operation.
    Extraction {
        /// The path to the backup.
        backup_path: PathBuf,
        /// The path in which to place the extracted files and directories.
        output_path: PathBuf,
        /// The number of threads to spawn in the thread pool.
        pool_size: u8,
        /// The backup password.
        password: String,
    },
}

impl Operation {
    /// Is this a backup operation?
    pub const fn is_backup(&self) -> bool {
        matches!(
            self,
            Self::Backup {
                include_paths: _,
                output_path: _,
                exclude_globs: _,
                chunk_size_magnitude: _,
                pool_size: _,
                password: _,
            }
        )
    }

    /// Is this an extraction operation?
    #[allow(dead_code)]
    pub const fn is_extraction(&self) -> bool {
        matches!(
            self,
            Self::Extraction {
                backup_path: _,
                output_path: _,
                pool_size: _,
                password: _,
            }
        )
    }

    /// Perform the configured operation.
    pub fn execute(self) -> BackupResult<PathBuf> {
        match self {
            Self::Backup {
                include_paths,
                output_path,
                exclude_globs,
                chunk_size_magnitude,
                pool_size,
                password,
            } => perform_backup(
                include_paths,
                output_path,
                exclude_globs,
                chunk_size_magnitude,
                pool_size,
                password,
            ),
            Self::Extraction {
                backup_path,
                output_path,
                pool_size,
                password,
            } => perform_extraction(backup_path, output_path, pool_size, password),
        }
    }
}
