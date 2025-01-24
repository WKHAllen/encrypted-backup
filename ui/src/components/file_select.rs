//! File selection component.

use super::ControlError;
use crate::classes::*;
use crate::hooks::*;
use dioxus::prelude::*;
use std::path::PathBuf;

/// File selection component.
#[component]
pub fn FileSelect(
    /// The file selection state.
    state: Signal<Option<PathBuf>>,
    /// The label text.
    label: Option<String>,
    /// Additional control info.
    info: Option<String>,
    /// The browse button label.
    browse_label: Option<String>,
    /// The path at which to start the selection.
    start_path: Option<PathBuf>,
    /// Text to display when no path is selected.
    empty_text: Option<String>,
    /// Whether the selection should allow directories instead of files.
    #[props(default = false)]
    directory: bool,
    /// An optional class name.
    class: Option<String>,
    /// An optional error message.
    #[props(!optional, default)]
    error: Option<String>,
) -> Element {
    let id = use_id();
    let label = label.unwrap_or_default();
    let info = info.unwrap_or_default();
    let display_text = state.with(|maybe_path| match maybe_path {
        Some(path) => path.display().to_string(),
        None => empty_text.unwrap_or_else(|| "No path selected".to_owned()),
    });
    let browse_label = browse_label.unwrap_or_else(|| "Browse".to_owned());

    rsx! {
        div {
            class: classes!("file-select-container", class),

            span {
                class: "file-select-label",
                "{label}"
            }

            div {
                class: "file-select",

                div {
                    class: "file-select-display",
                    "{display_text}"
                }

                label {
                    class: "file-select-button primary",
                    r#for: "{id}",
                    "{browse_label}"
                }

                input {
                    id: "{id}",
                    class: "file-select-input",
                    r#type: "file",
                    directory: directory,
                    onchange: move |event| {
                        if let Some(file_engine) = event.files() {
                            if let Some(path) = file_engine.files().first() {
                                state.set(Some(PathBuf::from(path)));
                            }
                        }
                    }
                }
            }

            span {
                class: "file-select-info",
                "{info}"
            }

            ControlError {
                message: error
            }
        }
    }
}
