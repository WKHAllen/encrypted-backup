//! Extraction operation configuration.

use dioxus::prelude::*;

/// The extraction operation configuration component.
#[component]
pub fn ExtractionConfig() -> Element {
    rsx! {
        div {
            class: "extraction-config",

            "Extraction configuration"

            // BASIC CONFIG OPTIONS:
            // backup_path: PathBuf,
            // output_path: PathBuf,

            // ADVANCED CONFIG OPTIONS:
            // pool_size: u8,

            // PROMPT IN POPUP ON EXTRACTION START:
            // password: Option<String>,

            // REMOVE OPTION AND DISPLAY CONFIRMATION POPUP IF OVER SUGGESTED MEMORY LIMIT:
            // override_memory_limit: bool,

            // REMOVE OPTION AND ALWAYS SHOW DEBUG LOG:
            // debug: bool,
        }
    }
}
