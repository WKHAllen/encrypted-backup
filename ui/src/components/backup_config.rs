//! Backup operation configuration.

use super::{ExcludeGlobs, FileSelect, Icon, IncludePathsSelect, Slider};
use crate::classes::*;
use crate::icons::*;
use backup::{estimated_memory_usage, format_bytes, MEMORY_LIMIT};
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
    let mut basic_config_open = use_signal(|| true);
    let mut advanced_config_open = use_signal(|| false);

    let chunk_size = 1 << chunk_size_magnitude();
    let memory_usage_estimate = estimated_memory_usage(chunk_size, pool_size());
    let over_memory_limit = memory_usage_estimate > MEMORY_LIMIT;

    rsx! {
        div {
            class: "backup-config",

            // BASIC CONFIG OPTIONS
            div {
                class: "config-title-container",

                div {
                    class: "config-title",
                    onclick: move |_| {
                        basic_config_open.set(!basic_config_open());
                    },

                    Icon {
                        data: CARET_UP,
                        class: classes!("config-section-icon", basic_config_open().then_some("config-section-icon-open")),
                    }

                    h2 {
                        class: "config-title-text",
                        "Basic configuration"
                    }
                }
            }

            div {
                class: classes!("config-section", basic_config_open().then_some("config-section-open")),

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
            }

            // ADVANCED CONFIG OPTIONS
            div {
                class: "config-title-container",

                div {
                    class: "config-title",
                    onclick: move |_| {
                        advanced_config_open.set(!advanced_config_open());
                    },

                    Icon {
                        data: CARET_UP,
                        class: classes!("config-section-icon", advanced_config_open().then_some("config-section-icon-open")),
                    }

                    h2 {
                        class: "config-title-text",
                        "Advanced configuration"
                    }
                }
            }

            div {
                class: classes!("config-section", advanced_config_open().then_some("config-section-open")),

                // exclude_globs: Vec<Pattern>
                ExcludeGlobs {
                    state: exclude_globs,
                }

                // chunk_size_magnitude: u8
                Slider {
                    state: chunk_size_magnitude,
                    label: "Chunk size magnitude",
                    info: format!("The backup will be encoded in chunks of size {}", format_bytes(1 << chunk_size_magnitude())),
                    min: 12,
                    max: 28,
                    step: 1,
                }

                // pool_size: u8
                Slider {
                    state: pool_size,
                    label: "Pool size",
                    info: "This determines how many workers to spawn in a pool that will perform cryptographic operations in parallel",
                    min: 1,
                    max: 24,
                    step: 1,
                }

                // ESTIMATED MEMORY USAGE
                div {
                    span {
                        class: "estimated-memory-usage-message",
                        "Estimated memory usage: "
                    }

                    span {
                        class: classes!("estimated-memory-usage-amount", over_memory_limit.then_some("estimated-memory-usage-amount-large")),
                        "{format_bytes(memory_usage_estimate)}"
                    }
                }
            }

            // PROMPT IN POPUP ON BACKUP START
            // password: Option<String>

            // REMOVE OPTION AND DISPLAY CONFIRMATION POPUP IF OVER SUGGESTED MEMORY LIMIT
            // override_memory_limit: bool

            // REMOVE OPTION AND ALWAYS SHOW DEBUG LOG
            // debug: bool
        }
    }
}
