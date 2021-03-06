use crate::backup_crypto::*;
use crate::types::*;
use glob::Pattern;
use log::info;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str;
use tempfile::NamedTempFile;

/// Turn a password into a 256-bit key.
///
/// `password`: the password.
///
/// Returns the key generated from the hash of the password.
fn password_to_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password);
    (&hasher.finalize()[..32]).try_into().unwrap()
}

/// Check if a path is excluded based on a list of globs.
///
/// `path`: the path.
/// `exclude_globs`: the list of globs.
///
/// Returns whether the path is excluded by one or more of the globs.
fn glob_excluded(path: &Path, exclude_globs: &[Pattern]) -> bool {
    for glob in exclude_globs {
        if glob.matches_path(&path) {
            return true;
        }
    }

    false
}

/// Get the last component of a path.
///
/// `path`: the path.
///
/// Returns an option containing the last component of the path, or None if the path is empty.
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

/// Check that there are no duplicate names in the paths included in a backup.
///
/// `include_paths`: the list of paths to be included in the backup.
///
/// Returns a result of the error variant if a duplicate include name was found.
fn validate_no_duplicate_include_names(include_paths: &[PathBuf]) -> BackupResult<()> {
    // Use a HashSet for quick lookups
    let mut include_set = HashSet::new();

    // Check all include paths for duplicates
    for include_path in include_paths {
        // The "name" of the include path is determined by the last component of its path
        let include_name = last_path_component(include_path).unwrap();

        // If an include path with the same name is already in the set, then we have a duplicate
        if include_set.contains(&include_name) {
            return Err(BackupError::DuplicateIncludeName(include_name));
        } else {
            include_set.insert(include_name);
        }
    }

    // No duplicates
    Ok(())
}

/// Check that a path does not already exist.
///
/// `path`: the path.
/// `path_type`: the type of path to check.
///
/// Returns a result of the error variant if the path already exists.
fn validate_path_does_not_exist(path: &Path, path_type: PathType) -> BackupResult<()> {
    if path.exists() {
        match path_type {
            PathType::File => {
                if path.is_file() {
                    Err(BackupError::PathAlreadyExists(path.to_path_buf()))
                } else {
                    Ok(())
                }
            }
            PathType::Directory => {
                if path.is_dir() {
                    Err(BackupError::PathAlreadyExists(path.to_path_buf()))
                } else {
                    Ok(())
                }
            }
            PathType::Any => Err(BackupError::PathAlreadyExists(path.to_path_buf())),
        }
    } else {
        Ok(())
    }
}

/// Append files to a tar archive recursively.
///
/// `archive`: a mutable reference to the tar archive.
/// `include_path`: the path to the directory to be included in the archive.
/// `exclude_globs`: the list of globs to exclude from the archive.
/// `relative_path`: the relative path from the root of the archive.
///
/// Returns a result of the error variant if an error occurred while adding to the archive.
fn append_to_archive<T: io::Write>(
    archive: &mut tar::Builder<T>,
    include_path: &Path,
    exclude_globs: &[Pattern],
    relative_path: &Path,
) -> io::Result<()> {
    if !glob_excluded(&relative_path, &exclude_globs) {
        if include_path.is_dir() {
            // Append the directory itself (this is necessary because if the directory is empty, it will not be appended to the archive)
            match archive.append_path_with_name(&include_path, &relative_path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Ok(()),
                Err(e) => Err(e),
            }?;

            // Read the list of entries in the directory
            let entries = match fs::read_dir(&include_path) {
                Ok(val) => Ok(val),
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Ok(()),
                Err(e) => Err(e),
            }?;

            // Iterate over all entries that did not throw errors
            for entry in entries.into_iter().filter_map(|e| e.ok()) {
                let entry_path = include_path.join(entry.file_name().to_str().unwrap());
                let entry_relative_path = relative_path.join(entry.file_name().to_str().unwrap());

                // Recursively call this function for the current directory entry to add all of its contents to the archive
                append_to_archive(archive, &entry_path, exclude_globs, &entry_relative_path)?;
            }
        } else if include_path.is_file() {
            // Add the current file entry to the archive
            match archive.append_path_with_name(include_path, relative_path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Ok(()),
                Err(e) => Err(e),
            }?;
        }
    }

    Ok(())
}

/// Back up and encrypt a set of paths.
///
/// `include_paths`: the list of paths to back up.
/// `exclude_globs`: the list of globs to exclude from the backup.
/// `output_path`: the path to save the backup to.
/// `password`: the password used to encrypt the backup.
///
/// Returns a result containing the path to the encrypted backup, or the error variant if an error occurred while performing the backup.
pub fn backup(
    include_paths: &[PathBuf],
    exclude_globs: &[Pattern],
    output_path: &Path,
    password: &str,
) -> BackupResult<PathBuf> {
    info!("Validating backup");

    // Make sure there are no include directories with the same name
    validate_no_duplicate_include_names(include_paths)?;

    // Make sure output file does not already exist
    validate_path_does_not_exist(&output_path, PathType::Any)?;

    info!("Beginning backup");

    // Create the tar archive
    let tar_file = NamedTempFile::new()?;
    let tar_path = tar_file.path();
    let mut archive = tar::Builder::new(&tar_file);

    // Add each include path to the archive
    for include_path in include_paths {
        info!("Backing up '{}'", include_path.display());

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

    info!("Encrypting backup");

    // Turn the password into a 256-bit key used for encryption
    let key = password_to_key(&password);

    // Read and encrypt the tar archive
    encrypt_backup(&tar_path, &output_path, &key)?;

    // Delete temporary tar file
    fs::remove_file(&tar_path)?;

    info!("Backup complete");

    // Return the output file path
    Ok(output_path.to_path_buf())
}

/// Extract an encrypted backup.
///
/// `path`: the path to the encrypted backup.
/// `output_path`: the path to extract the backup to.
/// `password`: the password used to decrypt the backup.
///
/// Returns a result containing the path to the extracted backup, or the error variant if an error occurred while performing the extraction.
pub fn extract(path: &Path, output_path: &Path, password: &str) -> BackupResult<PathBuf> {
    info!("Validating extraction");

    // Make sure output directory does not already exist
    validate_path_does_not_exist(&output_path, PathType::Any)?;

    info!("Decrypting backup");

    // Turn the password into a 256-bit key used for encryption
    let key = password_to_key(&password);

    // Decrypt the backup
    let tar_file = decrypt_backup(&path, &key)?;

    info!("Extracting decrypted backup");

    // Extract the tar file
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&output_path)?;

    info!("Extraction complete");

    // Return the output directory path
    Ok(output_path.to_path_buf())
}
