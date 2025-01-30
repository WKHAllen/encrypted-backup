//! Extraction operation configuration.

use super::{FileSelect, Icon, Slider};
use crate::classes::*;
use crate::icons::CARET_UP;
use backup::{backup_chunk_size, estimated_memory_usage, format_bytes, MEMORY_LIMIT};
use dioxus::prelude::*;

/// The extraction operation configuration component.
#[component]
pub fn ExtractionConfig() -> Element {
    let backup_path = use_signal(|| None);
    let backup_path_error = use_signal(|| None);
    let output_path = use_signal(|| None);
    let output_path_error = use_signal(|| None);
    let pool_size = use_signal(|| 4u8);
    let mut basic_config_open = use_signal(|| true);
    let mut advanced_config_open = use_signal(|| false);

    let memory_usage_estimate = backup_path.with(|path| match path {
        Some(path) => match backup_chunk_size(path) {
            Ok(chunk_size) => Some(estimated_memory_usage(chunk_size, pool_size())),
            Err(_) => None,
        },
        None => None,
    });
    let over_memory_limit = memory_usage_estimate.is_some_and(|estimate| estimate > MEMORY_LIMIT);

    rsx! {
        div {
            class: "extraction-config",

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

                // backup_path: PathBuf
                FileSelect {
                    state: backup_path,
                    label: "Backup path",
                    empty_text: "No backup path selected",
                    error: backup_path_error(),
                }

                // output_path: PathBuf
                FileSelect {
                    state: output_path,
                    label: "Output path",
                    info: "This is the directory in which the contents of the backup file will be extracted",
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
                match memory_usage_estimate {
                    Some(estimate) => rsx! {
                        div {
                            span {
                                class: "estimated-memory-usage-message",
                                "Estimated memory usage: "
                            }

                            span {
                                class: classes!("estimated-memory-usage-amount", over_memory_limit.then_some("estimated-memory-usage-amount-large")),
                                "{format_bytes(estimate)}"
                            }
                        }
                    },
                    None => rsx! {
                        div {
                            span {
                                class: "estimated-memory-usage-message",
                                "Memory usage estimate is not possible because a backup path has not been selected or could not be read"
                            }
                        }
                    }
                }
            }

            // PROMPT IN POPUP ON EXTRACTION START
            // password: Option<String>

            // REMOVE OPTION AND DISPLAY CONFIRMATION POPUP IF OVER SUGGESTED MEMORY LIMIT
            // override_memory_limit: bool

            // REMOVE OPTION AND ALWAYS SHOW DEBUG LOG
            // debug: bool
        }
    }
}
