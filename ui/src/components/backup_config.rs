//! Backup operation configuration.

use super::{FileSelect, IncludePathsSelect};
use dioxus::prelude::*;

/// The backup operation configuration component.
#[component]
pub fn BackupConfig() -> Element {
    let include_paths = use_signal(Vec::new);
    let output_path = use_signal(|| None);
    let output_path_error = use_signal(|| None);

    rsx! {
        div {
            class: "backup-config",

            // BASIC CONFIG OPTIONS
            h2 {
                class: "config-title",
                "Basic configuration"
            }

            // include_paths: Vec<PathBuf>
            IncludePathsSelect {
                state: include_paths,
            }

            // output_path: PathBuf
            FileSelect {
                state: output_path,
                label: "Output path",
                info: "This is the directory in which the backup file will be created",
                empty_text: "No output path selected",
                directory: true,
                error: output_path_error(),
            }

            // ADVANCED CONFIG OPTIONS
            h2 {
                class: "config-title",
                "Advanced configuration"
            }

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
