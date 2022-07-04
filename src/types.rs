use std::fmt;
use std::io;
use std::path::PathBuf;

/// An error during a backup or extraction.
pub enum BackupError {
    IOError(io::Error),
    CryptoError(aes_gcm::Error),
    DuplicateIncludeName(String),
    PathAlreadyExists(PathBuf),
    BackupReadFailed(String),
}

impl BackupError {
    pub fn msg(&self) -> String {
        match self {
            Self::IOError(e) => format!("IO error: {}", e),
            Self::CryptoError(e) => format!("Crypto error: {}", e),
            Self::DuplicateIncludeName(name) => format!("Duplicate include path name: {}", name),
            Self::PathAlreadyExists(path) => format!("Path already exists: {} ", path.display()),
            Self::BackupReadFailed(msg) => format!("Backup read failed: {}", msg),
        }
    }
}

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.msg().as_str())
    }
}

impl From<io::Error> for BackupError {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<aes_gcm::Error> for BackupError {
    fn from(e: aes_gcm::Error) -> Self {
        Self::CryptoError(e)
    }
}

pub type BackupResult<T> = Result<T, BackupError>;

#[allow(dead_code)]
pub enum PathType {
    File,
    Directory,
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
