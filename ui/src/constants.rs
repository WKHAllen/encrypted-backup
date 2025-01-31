//! Global application constants.

use std::time::Duration;

/// Whether this is a debug build.
pub const DEBUG: bool = cfg!(debug_assertions);

/// The application window title.
pub const WINDOW_TITLE: &str = "Encrypted Backup";

// /// The application window icon.
// pub const WINDOW_ICON: &[u8] = include_bytes!("../assets/img/icon.ico");

/// The name of the configuration file.
pub const CONFIG_FILE_NAME: &str = "config.json";

/// The duration of time to wait before saving the configuration file.
pub const SAVE_CONFIG_SLEEP_DURATION: Duration = Duration::from_secs(2);
