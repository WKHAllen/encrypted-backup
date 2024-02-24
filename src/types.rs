//! Application-level type definitions.

use std::fmt;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// An error during a backup or extraction.
#[derive(Debug, Error)]
pub enum BackupError {
    /// An I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    /// A cryptographic error.
    #[error("cryptographic error: {0}")]
    CryptoError(aes_gcm::Error),
    /// A duplicate include path name was encountered.
    #[error("duplicate include path name: {0}")]
    DuplicateIncludeName(String),
    /// The specified path already exists.
    #[error("path already exists: {0}")]
    PathAlreadyExists(PathBuf),
}

impl From<aes_gcm::Error> for BackupError {
    fn from(e: aes_gcm::Error) -> Self {
        Self::CryptoError(e)
    }
}

/// An application-level backup-related `Result`.
pub type BackupResult<T> = Result<T, BackupError>;

/// A type of path.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum PathType {
    /// A file.
    File,
    /// A directory.
    Directory,
    /// Either a file or a directory.
    Any,
}

impl fmt::Display for PathType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::Any => "file or directory",
        })
    }
}
