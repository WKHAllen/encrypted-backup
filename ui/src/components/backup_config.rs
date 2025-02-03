//! Backup operation configuration.

use super::{ExcludeGlobs, FileSelect, Icon, IncludePathsSelect, Slider};
use crate::classes::*;
use crate::constants::*;
use crate::icons::*;
use crate::services::{
    parse_pattern, BackupConfig as BackupConfigState, Config as ConfigState, Operation,
};
use backup::{estimated_memory_usage, format_bytes, MEMORY_LIMIT};
use dioxus::prelude::*;
use glob::{Pattern, PatternError};
use std::rc::Rc;
use tokio::time::sleep;

/// The backup operation configuration component.
#[component]
pub fn BackupConfig(
    /// Is this configuration currently active?
    active: bool,
    /// The initial configuration.
    config: BackupConfigState,
    /// The callback to execute when ready to perform an operation.
    start: EventHandler<Operation>,
) -> Element {
    let include_paths = use_signal(|| config.include_paths);
    let output_path = use_signal(|| config.output_path);
    let output_path_error = use_signal(|| None);
    let exclude_globs = use_signal(|| config.exclude_globs.iter().map(parse_pattern).collect());
    let chunk_size_magnitude = use_signal(|| config.chunk_size_magnitude);
    let pool_size = use_signal(|| config.pool_size);
    let mut basic_config_open = use_signal(|| config.basic_config_open);
    let mut advanced_config_open = use_signal(|| config.advanced_config_open);

    let chunk_size = 1 << chunk_size_magnitude();
    let memory_usage_estimate = estimated_memory_usage(chunk_size, pool_size());
    let over_memory_limit = memory_usage_estimate > MEMORY_LIMIT;

    let output_path_specified = output_path.with(Option::is_some);
    let exclude_globs_ok =
        exclude_globs.with(|globs: &Vec<Result<Pattern, (String, Rc<PatternError>)>>| {
            globs.iter().all(Result::is_ok)
        });
    let form_valid = output_path_specified && exclude_globs_ok;
    let form_invalid_message = if !output_path_specified {
        "Cannot start a backup without selecting an output path"
    } else if !exclude_globs_ok {
        "Cannot start a backup with invalid exclusion glob patterns"
    } else {
        ""
    };

    let mut save_task = use_signal(|| None);

    let save_config = move |include_paths,
                            output_path,
                            exclude_globs: Vec<Result<Pattern, (String, Rc<PatternError>)>>,
                            chunk_size_magnitude,
                            pool_size,
                            basic_config_open,
                            advanced_config_open| {
        spawn(async move {
            let _ = async move {
                let mut config = ConfigState::load().await?;

                config.backup_config.include_paths = include_paths;
                config.backup_config.output_path = output_path;
                config.backup_config.exclude_globs = exclude_globs
                    .into_iter()
                    .map(|maybe_pattern| match maybe_pattern {
                        Ok(pattern) => pattern.as_str().to_owned(),
                        Err((invalid_pattern, _)) => invalid_pattern,
                    })
                    .collect();
                config.backup_config.chunk_size_magnitude = chunk_size_magnitude;
                config.backup_config.pool_size = pool_size;
                config.backup_config.basic_config_open = basic_config_open;
                config.backup_config.advanced_config_open = advanced_config_open;

                config.save().await
            }
            .await;
        });
    };

    use_effect(move || {
        let include_paths = include_paths();
        let output_path = output_path();
        let exclude_globs = exclude_globs();
        let chunk_size_magnitude = chunk_size_magnitude();
        let pool_size = pool_size();
        let basic_config_open = basic_config_open();
        let advanced_config_open = advanced_config_open();

        let previous_task = save_task.replace(Some(spawn(async move {
            sleep(SAVE_CONFIG_SLEEP_DURATION).await;
            save_config(
                include_paths,
                output_path,
                exclude_globs,
                chunk_size_magnitude,
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
            class: classes!("backup-config", (!active).then_some("backup-config-hidden")),

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

            div {
                class: "start",

                div {
                    button {
                        r#type: "button",
                        class: "button primary start-button",
                        disabled: !form_valid,
                        onclick: move |_| {
                            start(Operation::Backup {
                                include_paths: include_paths(),
                                output_path: output_path().unwrap(),
                                exclude_globs: exclude_globs().into_iter().map(|x| x.unwrap()).collect(),
                                chunk_size_magnitude: chunk_size_magnitude(),
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

            // TODO: PROMPT IN POPUP ON BACKUP START
            // password: Option<String>

            // TODO: REMOVE OPTION AND DISPLAY CONFIRMATION POPUP IF OVER SUGGESTED MEMORY LIMIT
            // override_memory_limit: bool
        }
    }
}
