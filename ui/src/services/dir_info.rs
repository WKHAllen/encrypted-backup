//! Directory information services.

use std::env;
use std::io;
use std::path::{Path, PathBuf};
use sysinfo::Disks;
use tokio::fs;
use tokio::task::spawn_blocking;

/// Information on a directory.
#[derive(Debug, Clone, PartialEq)]
pub struct DirectoryInfo {
    /// The directory's path.
    pub path: PathBuf,
    /// The list of subdirectories.
    pub dirs: Vec<String>,
    /// The list of files within the directory.
    pub files: Vec<String>,
}

/// Gets information on a directory that isn't at the root of the filesystem.
async fn get_non_root_dir_info<P>(path: P) -> io::Result<DirectoryInfo>
where
    P: AsRef<Path>,
{
    let mut dirs = vec![];
    let mut files = vec![];

    let mut entries = fs::read_dir(&path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;

        if file_type.is_dir() {
            dirs.push(entry.file_name().to_str().unwrap().to_owned());
        } else if file_type.is_file() {
            files.push(entry.file_name().to_str().unwrap().to_owned());
        }
    }

    Ok(DirectoryInfo {
        path: path.as_ref().to_path_buf(),
        dirs,
        files,
    })
}

/// Gets information on the directory at the root of the filesystem.
async fn get_root_dir_info() -> io::Result<DirectoryInfo> {
    match env::consts::OS {
        "windows" => {
            let disks_handle = spawn_blocking(|| {
                Disks::new_with_refreshed_list()
                    .into_iter()
                    .map(|disk| disk.mount_point().to_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            });
            let mut disks = disks_handle.await.unwrap();
            disks.sort();

            Ok(DirectoryInfo {
                path: PathBuf::from("/"),
                dirs: disks,
                files: vec![],
            })
        }
        _ => get_non_root_dir_info(Path::new("/")).await,
    }
}

/// Gets the directory information for any given path.
async fn collect_dir_info<P>(path: P) -> io::Result<DirectoryInfo>
where
    P: AsRef<Path>,
{
    match path.as_ref().to_str().unwrap() {
        "" | "/" => get_root_dir_info().await,
        _ => get_non_root_dir_info(path).await,
    }
}

/// Gets information on a directory.
pub async fn get_directory_info<P>(path: P) -> io::Result<DirectoryInfo>
where
    P: AsRef<Path>,
{
    let path = if !path.as_ref().ends_with("/") && !path.as_ref().ends_with("\\") {
        path.as_ref().join(Path::new(""))
    } else {
        path.as_ref().to_path_buf()
    };

    collect_dir_info(&path).await
}

/// Attempts to get the system's home directory.
pub async fn get_home_directory() -> Option<PathBuf> {
    spawn_blocking(home::home_dir).await.unwrap()
}
