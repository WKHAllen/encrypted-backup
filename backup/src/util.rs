//! Application-level utility functions.

use std::path::{Path, PathBuf};

/// Returns the provided path with `.tmp` added to it.
pub fn tmp_file_for(path: impl AsRef<Path>) -> PathBuf {
    let mut tmp_path = path.as_ref().to_path_buf();
    tmp_path.as_mut_os_string().push(".tmp");
    tmp_path
}
