//! File selection dialog component.

use crate::classes::*;
use crate::hooks::*;
use dioxus::prelude::*;
use std::path::PathBuf;

/// File selection dialog component.
#[component]
pub fn FileDialog(
    /// Whether the selection should allow directories instead of files.
    #[props(default = false)]
    directory: bool,
    /// The dialog open button children.
    children: Element,
    /// An optional class name.
    class: Option<String>,
    /// Event handler for when a file is selected.
    onselect: EventHandler<Option<PathBuf>>,
) -> Element {
    let id = use_id();

    rsx! {
        div {
            class: classes!("file-dialog", class),

            label {
                class: "file-dialog-label",
                r#for: "{id}",
                {children}
            }

            input {
                id: "{id}",
                class: "file-dialog-input",
                r#type: "file",
                directory: directory,
                onchange: move |event| {
                    if let Some(file_engine) = event.files() {
                        if let Some(path) = file_engine.files().first() {
                            onselect.call(Some(PathBuf::from(path)));
                        } else {
                            onselect.call(None);
                        }
                    }
                }
            }
        }
    }
}
