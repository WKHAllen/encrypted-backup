use super::crypto;
use glob::Pattern;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

pub enum BackupError {
    IOError(io::Error),
    CryptoError(aes_gcm::Error),
    DuplicateIncludeName(String),
    FileAlreadyExists(PathBuf),
}

impl BackupError {
    pub fn msg(&self) -> String {
        match self {
            Self::IOError(e) => format!("IO error: {}", e),
            Self::CryptoError(e) => format!("Crypto error: {}", e),
            Self::DuplicateIncludeName(name) => format!("Duplicate include path name: {}", name),
            Self::FileAlreadyExists(path) => format!("File already exists: {} ", path.display()),
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

fn password_to_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password);
    (&hasher.finalize()[..32]).try_into().unwrap()
}

fn glob_excluded(path: &Path, exclude_globs: &[Pattern]) -> bool {
    for glob in exclude_globs {
        if glob.matches_path(&path) {
            return true;
        }
    }

    false
}

fn last_path_component(path: &Path) -> Option<String> {
    Some(
        path.components()
            .last()?
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned(),
    )
}

fn validate_no_duplicate_include_names(include_paths: &[PathBuf]) -> Result<(), BackupError> {
    let mut include_set = HashSet::new();

    for include_path in include_paths {
        let include_name = last_path_component(include_path).unwrap();

        if include_set.contains(&include_name) {
            return Err(BackupError::DuplicateIncludeName(include_name));
        } else {
            include_set.insert(include_name);
        }
    }

    Ok(())
}

fn validate_file_does_not_exist(path: &Path) -> Result<(), BackupError> {
    if path.exists() {
        Err(BackupError::FileAlreadyExists(path.to_path_buf()))
    } else {
        Ok(())
    }
}

fn append_to_archive<T: io::Write>(
    archive: &mut tar::Builder<T>,
    include_path: &Path,
    exclude_globs: &[Pattern],
    relative_path: &Path,
) -> io::Result<()> {
    let entries = fs::read_dir(&include_path)?;

    for entry in entries.into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().unwrap().is_dir() {
            let entry_path = include_path.join(entry.file_name().to_str().unwrap());
            let entry_relative_path = relative_path.join(entry.file_name().to_str().unwrap());

            if !glob_excluded(&entry_path, &exclude_globs) {
                append_to_archive(archive, &entry_path, exclude_globs, &entry_relative_path)?;
            }
        } else if entry.file_type().unwrap().is_file() {
            let entry_path = include_path.join(entry.file_name().to_str().unwrap());
            let entry_relative_path = relative_path.join(entry.file_name().to_str().unwrap());

            if !glob_excluded(&entry_path, &exclude_globs) {
                archive.append_path_with_name(entry_path, entry_relative_path)?;
            }
        }
    }

    Ok(())
}

/// Back up and encrypt a set of paths.
///
/// include_paths: a list of paths to back up.
/// exclude_globs: a list of globs to exclude from the backup.
/// output_dir: the directory in which to save the backup.
/// name: the name of the backup.
/// password: the password used to encrypt the backup.
///
/// Returns a result containing the path to the encrypted backup, or the error variant if an error occurred while performing the backup.
pub fn backup(
    include_paths: &[PathBuf],
    exclude_globs: &[Pattern],
    output_dir: &Path,
    name: &str,
    password: &str,
) -> Result<PathBuf, BackupError> {
    // Make sure there are no include directories with the same name
    validate_no_duplicate_include_names(include_paths)?;

    // Make sure tar and output files do not already exist
    let tar_path = output_dir.join(format!("{}.tar", name));
    let encrypted_path = output_dir.join(format!("{}.backup", name));
    validate_file_does_not_exist(&tar_path)?;
    validate_file_does_not_exist(&encrypted_path)?;

    // Create the tar archive
    let tar_file = File::create(&tar_path)?;
    let mut archive = tar::Builder::new(tar_file);

    // Add each include path to the archive
    for include_path in include_paths {
        let include_name = last_path_component(include_path).unwrap();

        append_to_archive(
            &mut archive,
            &include_path,
            &exclude_globs,
            Path::new(&include_name),
        )?;
    }

    // Close the archive
    archive.finish()?;

    // Turn the password into a 256-bit key used for encryption
    let key = password_to_key(&password);

    // Read and encrypt the tar archive
    let tar_data = fs::read(&tar_path)?;
    let encrypted_data = crypto::aes_encrypt(&key, &tar_data)?;

    // Write the encrypted data to the output file and delete the tar archive
    fs::write(&encrypted_path, encrypted_data)?;
    fs::remove_file(&tar_path)?;

    // Return the output file path
    Ok(encrypted_path)
}

/// Extract an encrypted backup.
///
/// path: the path to the encrypted backup.
/// password: the password used to decrypt the backup.
///
/// Returns a result containing the path to the extracted backup, or the error variant if an error occurred while performing the extraction.
pub fn extract(path: &Path, password: &str) -> Result<PathBuf, BackupError> {
    // TODO: decrypt file
    // TODO: extract tar file to directory

    todo!()
}
