//! API to trigger a backup or extraction operation.

use backup::BackupResult;
use glob::Pattern;
use std::path::PathBuf;

/// Perform the configured backup operation.
fn perform_backup(
    include_paths: Vec<PathBuf>,
    output_path: PathBuf,
    exclude_globs: Vec<Pattern>,
    chunk_size_magnitude: u8,
    pool_size: u8,
) -> BackupResult<PathBuf> {
    todo!()
}

/// Perform the configured extraction operation.
fn perform_extraction(
    backup_path: PathBuf,
    output_path: PathBuf,
    pool_size: u8,
) -> BackupResult<PathBuf> {
    todo!()
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
    },
    /// An extraction operation.
    Extraction {
        /// The path to the backup.
        backup_path: PathBuf,
        /// The path in which to place the extracted files and directories.
        output_path: PathBuf,
        /// The number of threads to spawn in the thread pool.
        pool_size: u8,
    },
}

impl Operation {
    /// Perform the configured operation.
    pub fn execute(self) -> BackupResult<PathBuf> {
        match self {
            Self::Backup {
                include_paths,
                output_path,
                exclude_globs,
                chunk_size_magnitude,
                pool_size,
            } => perform_backup(
                include_paths,
                output_path,
                exclude_globs,
                chunk_size_magnitude,
                pool_size,
            ),
            Self::Extraction {
                backup_path,
                output_path,
                pool_size,
            } => perform_extraction(backup_path, output_path, pool_size),
        }
    }
}
