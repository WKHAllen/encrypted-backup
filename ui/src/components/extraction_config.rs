//! Extraction operation configuration.

use super::{Dialog, FileSelect, Icon, Slider};
use crate::classes::*;
use crate::constants::*;
use crate::icons::CARET_UP;
use crate::services::{
    Config as ConfigState, ExtractionConfig as ExtractionConfigState, Operation,
};
use backup::{backup_chunk_size, estimated_memory_usage, format_bytes, MEMORY_LIMIT};
use dioxus::prelude::*;
use tokio::time::sleep;

/// The extraction operation configuration component.
#[component]
pub fn ExtractionConfig(
    /// Is this configuration currently active?
    active: bool,
    /// The initial configuration.
    config: ExtractionConfigState,
    /// The callback to execute when ready to perform an operation.
    start: EventHandler<Operation>,
) -> Element {
    let backup_path = use_signal(|| config.backup_path);
    let backup_path_error = use_signal(|| None);
    let output_path = use_signal(|| config.output_path);
    let output_path_error = use_signal(|| None);
    let pool_size = use_signal(|| config.pool_size);
    let mut basic_config_open = use_signal(|| config.basic_config_open);
    let mut advanced_config_open = use_signal(|| config.advanced_config_open);

    let memory_usage_estimate = backup_path.with(|path| match path {
        Some(path) => match backup_chunk_size(path) {
            Ok(chunk_size) => Some(estimated_memory_usage(chunk_size, pool_size())),
            Err(_) => None,
        },
        None => None,
    });
    let over_memory_limit = memory_usage_estimate.is_some_and(|estimate| estimate > MEMORY_LIMIT);

    let backup_path_specified = backup_path.with(Option::is_some);
    let output_path_specified = output_path.with(Option::is_some);
    let form_valid = backup_path_specified && output_path_specified;
    let form_invalid_message = if !backup_path_specified {
        "Cannot start an extraction without selecting a backup path"
    } else if !output_path_specified {
        "Cannot start an extraction without selecting an output path"
    } else {
        ""
    };

    let mut save_task = use_signal(|| None);

    let save_config =
        move |backup_path, output_path, pool_size, basic_config_open, advanced_config_open| {
            spawn(async move {
                let _ = async move {
                    let mut config = ConfigState::load().await?;

                    config.extraction_config.backup_path = backup_path;
                    config.extraction_config.output_path = output_path;
                    config.extraction_config.pool_size = pool_size;
                    config.extraction_config.basic_config_open = basic_config_open;
                    config.extraction_config.advanced_config_open = advanced_config_open;

                    config.save().await
                }
                .await;
            });
        };

    use_effect(move || {
        let backup_path = backup_path();
        let output_path = output_path();
        let pool_size = pool_size();
        let basic_config_open = basic_config_open();
        let advanced_config_open = advanced_config_open();

        let previous_task = save_task.replace(Some(spawn(async move {
            sleep(SAVE_CONFIG_SLEEP_DURATION).await;
            save_config(
                backup_path,
                output_path,
                pool_size,
                basic_config_open,
                advanced_config_open,
            );
        })));

        if let Some(task) = previous_task {
            task.cancel();
        }
    });

    rsx! {
        div {
            class: classes!("extraction-config", (!active).then_some("extraction-config-hidden")),

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

            div {
                class: "start",

                div {
                    button {
                        r#type: "button",
                        class: "button primary start-button",
                        disabled: !form_valid,
                        onclick: move |_| {
                            start(Operation::Extraction {
                                backup_path: backup_path().unwrap(),
                                output_path: output_path().unwrap(),
                                pool_size: pool_size(),
                                password: String::new(),
                            });
                        },

                        "Start"
                    }
                }

                span {
                    class: "info",
                    "{form_invalid_message}"
                }
            }

            // TODO: REMOVE OPTION AND DISPLAY CONFIRMATION POPUP IF OVER SUGGESTED MEMORY LIMIT
            // override_memory_limit: bool

            // TODO: PROMPT IN POPUP ON EXTRACTION START
            // password: String
        }
    }
}
