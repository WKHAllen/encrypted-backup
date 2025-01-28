//! Backup operation configuration.

use super::{ExcludeGlobs, FileSelect, IncludePathsSelect, Slider};
use crate::format::*;
use dioxus::prelude::*;

/// The backup operation configuration component.
#[component]
pub fn BackupConfig() -> Element {
    let include_paths = use_signal(Vec::new);
    let output_path = use_signal(|| None);
    let output_path_error = use_signal(|| None);
    let exclude_globs = use_signal(Vec::new);
    let chunk_size_magnitude = use_signal(|| 16u8);
    let pool_size = use_signal(|| 4u8);

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
            ExcludeGlobs {
                state: exclude_globs,
            }

            // chunk_size_magnitude: u8,
            Slider {
                state: chunk_size_magnitude,
                label: "Chunk size magnitude",
                info: format!("The backup will be encoded in chunks of size {}", format_size(1 << chunk_size_magnitude())),
                min: 12,
                max: 28,
                step: 1,
            }

            // pool_size: u8,
            Slider {
                state: pool_size,
                label: "Pool size",
                info: "This determines how many workers to spawn in a pool that will perform cryptographic operations in parallel",
                min: 1,
                max: 24,
                step: 1,
            }

            // PROMPT IN POPUP ON BACKUP START
            // password: Option<String>,

            // REMOVE OPTION AND DISPLAY CONFIRMATION POPUP IF OVER SUGGESTED MEMORY LIMIT:
            // override_memory_limit: bool,

            // REMOVE OPTION AND ALWAYS SHOW DEBUG LOG:
            // debug: bool,

            // DISPLAY ESTIMATED MEMORY USAGE
        }
    }
}
