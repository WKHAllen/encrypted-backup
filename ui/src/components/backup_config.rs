//! Backup operation configuration.

use dioxus::prelude::*;

/// The backup operation configuration component.
#[component]
pub fn BackupConfig() -> Element {
    rsx! {
        div {
            class: "backup-config",

            "Backup configuration"

            // BASIC CONFIG OPTIONS:
            // include_paths: Vec<PathBuf>,
            // output_path: PathBuf,

            // ADVANCED CONFIG OPTIONS:
            // exclude_globs: Vec<Pattern>,
            // chunk_size_magnitude: u8,
            // pool_size: u8,

            // PROMPT IN POPUP ON BACKUP START
            // password: Option<String>,

            // REMOVE OPTION AND DISPLAY CONFIRMATION POPUP IF OVER SUGGESTED MEMORY LIMIT:
            // override_memory_limit: bool,

            // REMOVE OPTION AND ALWAYS SHOW DEBUG LOG:
            // debug: bool,
        }
    }
}
