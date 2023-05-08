use crate::backup_crypto::*;
use crate::crypto::*;
use crate::types::*;
use glob::Pattern;
use log::info;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str;
use tempfile::NamedTempFile;

/// Checks if a path is excluded based on a list of globs.
fn glob_excluded(path: impl AsRef<Path>, exclude_globs: &[Pattern]) -> bool {
    for glob in exclude_globs {
        if glob.matches_path(path.as_ref()) {
            return true;
        }
    }

    false
}

/// Gets the last component of a path.
fn last_path_component(path: impl AsRef<Path>) -> Option<String> {
    Some(
        path.as_ref()
            .components()
            .last()?
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned(),
    )
}

/// Checks that there are no duplicate names in the paths included in a backup.
fn validate_no_duplicate_include_names(include_paths: &[impl AsRef<Path>]) -> BackupResult<()> {
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

/// Checks that a path does not already exist.
fn validate_path_does_not_exist(path: impl AsRef<Path>, path_type: PathType) -> BackupResult<()> {
    if path.as_ref().exists() {
        match path_type {
            PathType::File => {
                if path.as_ref().is_file() {
                    Err(BackupError::PathAlreadyExists(path.as_ref().to_path_buf()))
                } else {
                    Ok(())
                }
            }
            PathType::Directory => {
                if path.as_ref().is_dir() {
                    Err(BackupError::PathAlreadyExists(path.as_ref().to_path_buf()))
                } else {
                    Ok(())
                }
            }
            PathType::Any => Err(BackupError::PathAlreadyExists(path.as_ref().to_path_buf())),
        }
    } else {
        Ok(())
    }
}

/// Appends files to a tar archive recursively.
fn append_to_archive<T: Write>(
    archive: &mut tar::Builder<T>,
    include_path: impl AsRef<Path>,
    exclude_globs: &[Pattern],
    relative_path: impl AsRef<Path>,
) -> io::Result<()> {
    if !glob_excluded(&relative_path, &exclude_globs) {
        if include_path.as_ref().is_dir() {
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
                let entry_path = include_path
                    .as_ref()
                    .join(entry.file_name().to_str().unwrap());
                let entry_relative_path = relative_path
                    .as_ref()
                    .join(entry.file_name().to_str().unwrap());

                // Recursively call this function for the current directory entry to add all of its contents to the archive
                append_to_archive(archive, &entry_path, exclude_globs, &entry_relative_path)?;
            }
        } else if include_path.as_ref().is_file() {
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

/// Backs up and encrypts a set of paths.
pub fn backup(
    include_paths: &[impl AsRef<Path>],
    exclude_globs: &[Pattern],
    output_path: impl AsRef<Path>,
    password: &str,
    chunk_size: usize,
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
        info!("Backing up '{}'", include_path.as_ref().display());

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
    encrypt_backup(&tar_path, &output_path, &key, chunk_size)?;

    // Delete temporary tar file
    fs::remove_file(&tar_path)?;

    info!("Backup complete");

    // Return the output file path
    Ok(output_path.as_ref().to_path_buf())
}

/// Extracts an encrypted backup.
pub fn extract(
    path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    password: &str,
) -> BackupResult<PathBuf> {
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
    Ok(output_path.as_ref().to_path_buf())
}

/// Backup tests.
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{DirEntry, File};

    fn non_existent_temp_file() -> PathBuf {
        let temp_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let path = temp_path.to_path_buf();
        temp_path.close().unwrap();
        path
    }

    fn io_error<T, E>(err: E) -> io::Result<T>
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Err(io::Error::new(io::ErrorKind::Other, err))
    }

    fn keep_entry(entry: &DirEntry, ignore_dir_names: &[&str], ignore_file_names: &[&str]) -> bool {
        let entry_file_name = entry.file_name();
        let entry_name = entry_file_name.to_str().unwrap();

        if entry.file_type().unwrap().is_dir() {
            if ignore_dir_names.contains(&entry_name) {
                return false;
            }
        }

        if entry.file_type().unwrap().is_file() {
            if ignore_file_names.contains(&entry_name) {
                return false;
            }
        }

        true
    }

    fn verify_identical_trees(
        a: impl AsRef<Path>,
        b: impl AsRef<Path>,
        check_root_name: bool,
        ignore_dir_names: &[&str],
        ignore_file_names: &[&str],
    ) -> io::Result<()> {
        let a = a.as_ref();
        let b = b.as_ref();

        if a.is_file() && b.is_file() {
            let a_name = a.file_name().unwrap().to_str().unwrap();
            let b_name = b.file_name().unwrap().to_str().unwrap();

            if !(ignore_file_names.contains(&a_name) || ignore_dir_names.contains(&b_name)) {
                if check_root_name && a_name != b_name {
                    return io_error(format!(
                        "mismatched file names: {} != {}",
                        a.display(),
                        b.display()
                    ));
                }

                let a_contents = fs::read(a)?;
                let b_contents = fs::read(b)?;

                if a_contents != b_contents {
                    return io_error(format!(
                        "mismatched file contents: {} != {}",
                        a.display(),
                        b.display()
                    ));
                }
            }
        } else if a.is_dir() && b.is_dir() {
            let a_name = a.file_name().unwrap().to_str().unwrap();
            let b_name = b.file_name().unwrap().to_str().unwrap();

            if !(ignore_file_names.contains(&a_name) || ignore_dir_names.contains(&b_name)) {
                if check_root_name && a_name != b_name {
                    return io_error(format!(
                        "mismatched directory names: {} != {}",
                        a.display(),
                        b.display()
                    ));
                }

                let a_entries = fs::read_dir(a)?.collect::<Result<Vec<_>, _>>()?;
                let a_entries = a_entries
                    .iter()
                    .filter(|entry| keep_entry(entry, ignore_dir_names, ignore_file_names))
                    .collect::<Vec<_>>();
                let b_entries = fs::read_dir(b)?.collect::<Result<Vec<_>, _>>()?;
                let b_entries = b_entries
                    .iter()
                    .filter(|entry| keep_entry(entry, ignore_dir_names, ignore_file_names))
                    .collect::<Vec<_>>();

                if a_entries.len() != b_entries.len() {
                    return io_error(format!(
                        "different number of contents: {} != {}",
                        a.display(),
                        b.display()
                    ));
                }

                for i in 0..a_entries.len() {
                    verify_identical_trees(
                        a_entries[i].path(),
                        b_entries[i].path(),
                        true,
                        ignore_dir_names,
                        ignore_file_names,
                    )?;
                }
            }
        } else {
            return io_error(format!(
                "file type mismatch: {} != {}",
                a.display(),
                b.display()
            ));
        }

        Ok(())
    }

    #[test]
    fn test_backup() {
        let root = project_root::get_project_root().unwrap();
        let include_paths = [&root];
        let exclude_globs = [Pattern::new("**/target").unwrap()];
        let ignore_dir_names = ["target"];
        let ignore_file_names = [];
        let backup_output_path = non_existent_temp_file();
        let extract_output_path = non_existent_temp_file();
        let extract_output_root = extract_output_path.join("encrypted-backup");
        let password = "password123";
        let chunk_size = 1024;

        backup(
            &include_paths,
            &exclude_globs,
            &backup_output_path,
            password,
            chunk_size,
        )
        .unwrap();
        extract(&backup_output_path, &extract_output_path, password).unwrap();

        verify_identical_trees(
            &root,
            &extract_output_root,
            false,
            &ignore_dir_names,
            &ignore_file_names,
        )
        .unwrap();

        fs::remove_file(&backup_output_path).unwrap();
        fs::remove_dir_all(&extract_output_path).unwrap();
    }

    #[test]
    fn test_backup_empty() {
        let src_path = non_existent_temp_file();
        let empty_file = src_path.join("empty_file.txt");
        let empty_dir = src_path.join("empty_dir");
        let include_paths = [&src_path];
        let exclude_globs = [];
        let ignore_dir_names = [];
        let ignore_file_names = [];
        let backup_output_path = non_existent_temp_file();
        let extract_output_path = non_existent_temp_file();
        let extract_output_root = extract_output_path.join(&src_path.file_name().unwrap());
        let password = "password123";
        let chunk_size = 1024;

        {
            fs::create_dir(&src_path).unwrap();
            File::create(&empty_file).unwrap();
            fs::create_dir(&empty_dir).unwrap();
        }

        backup(
            &include_paths,
            &exclude_globs,
            &backup_output_path,
            password,
            chunk_size,
        )
        .unwrap();
        extract(&backup_output_path, &extract_output_path, password).unwrap();

        verify_identical_trees(
            &src_path,
            &extract_output_root,
            false,
            &ignore_dir_names,
            &ignore_file_names,
        )
        .unwrap();

        fs::remove_dir_all(&src_path).unwrap();
        fs::remove_file(&backup_output_path).unwrap();
        fs::remove_dir_all(&extract_output_path).unwrap();
    }
}
