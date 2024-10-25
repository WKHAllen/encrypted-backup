//! Backup/extraction operation configuration.

use crate::classes::*;
use crate::components::{BackupConfig, ExtractionConfig};
use dioxus::prelude::*;

/// The currently selected operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum OperationType {
    /// An encrypted backup.
    Backup,
    /// An extraction of an encrypted backup.
    Extraction,
}

/// The backup and extract operation configuration component.
#[component]
pub fn Config() -> Element {
    let mut operation_type = use_signal(|| OperationType::Backup);

    let backup_tab_class = classes!(
        "config-tab",
        matches!(operation_type(), OperationType::Backup).then_some("config-tab-active")
    );
    let extraction_tab_class = classes!(
        "config-tab",
        matches!(operation_type(), OperationType::Extraction).then_some("config-tab-active")
    );

    rsx! {
        div {
            class: "config",

            div {
                class: "config-tabs",

                div {
                    class: "config-tabs-inner",

                    div {
                        class: backup_tab_class,
                        onclick: move |_| {
                            operation_type.set(OperationType::Backup);
                        },

                        "Backup"
                    }

                    div {
                        class: extraction_tab_class,
                        onclick: move |_| {
                            operation_type.set(OperationType::Extraction);
                        },

                        "Extract"
                    }
                }
            }

            div {
                class: "config-options",

                div {
                    class: "config-options-inner",

                    match operation_type() {
                        OperationType::Backup => rsx! {
                            BackupConfig {}
                        },
                        OperationType::Extraction => rsx! {
                            ExtractionConfig {}
                        },
                    }
                }
            }
        }
    }
}