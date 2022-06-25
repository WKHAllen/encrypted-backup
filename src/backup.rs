use glob::Pattern;
use std::collections::HashSet;
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

pub enum BackupError {
    IOError(io::Error),
    DuplicateIncludeName(String),
}

impl BackupError {
    pub fn msg(&self) -> String {
        match self {
            Self::IOError(e) => format!("IO error: {}", e),
            Self::DuplicateIncludeName(name) => format!("Duplicate include path name: {}", name),
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

pub fn backup(
    include_paths: &[PathBuf],
    exclude_globs: &[Pattern],
    output_dir: &Path,
    name: &str,
) -> Result<PathBuf, BackupError> {
    validate_no_duplicate_include_names(include_paths)?;

    let tar_path = output_dir.join(format!("{}.tar", name));
    let file = File::create(&tar_path)?;
    let mut archive = tar::Builder::new(file);

    for include_path in include_paths {
        let include_name = last_path_component(include_path).unwrap();

        append_to_archive(
            &mut archive,
            &include_path,
            &exclude_globs,
            Path::new(&include_name),
        )?;
    }

    archive.finish()?;

    // TODO: encrypt tar file
    // TODO: delete tar file

    Ok(tar_path) // TODO: return encrypted file path
}

pub fn extract(path: &Path) -> Result<PathBuf, BackupError> {
    todo!()
}
