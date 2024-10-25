//! File selection component.

use super::{Icon, Loading, PathDisplay};
use crate::icons::*;
use crate::services::*;
use dioxus::prelude::*;
use std::io;
use std::path::{Path, PathBuf, MAIN_SEPARATOR, MAIN_SEPARATOR_STR};
use std::rc::Rc;

/// The current state of the directory structure.
#[derive(Clone)]
enum DirectoryInfoState {
    /// Currently fetching directory information.
    Fetching,
    /// Done fetching directory information.
    Completed(DirectoryInfo),
    /// An error occurred while fetching directory information.
    Error(Rc<io::Error>),
}

/// File selection component.
#[component]
pub fn FileSelect(
    /// The path at which to start the selection.
    start_path: Option<PathBuf>,
    /// Whether the selection should allow directories instead of files.
    #[props(default = false)]
    directory: bool,
    /// Whether the selection should allow cancellation.
    #[props(default = false)]
    cancelable: bool,
    /// The path selection callback.
    on_select: EventHandler<PathBuf>,
    /// The cancellation callback.
    on_cancel: Option<EventHandler<()>>,
) -> Element {
    let mut current_path = use_signal(|| start_path.unwrap_or_else(|| Path::new("").to_path_buf()));
    let mut selection = use_signal(|| None);
    let mut selecting_directory = use_signal(|| None::<PathBuf>);
    let mut status = use_signal(|| DirectoryInfoState::Fetching);

    use_future(move || async move {
        if let Some(home_dir) = get_home_directory().await {
            current_path.set(home_dir);
        }
    });

    let mut set_selection = move |entry: &str, with_directory: bool| {
        let path = PathBuf::from(entry);

        if let DirectoryInfoState::Completed(_) = status() {
            if directory && with_directory {
                match selecting_directory() {
                    Some(path) if path.to_str() == Some(entry) => {
                        let path = current_path.cloned().join(entry);
                        current_path.set(path.clone());
                        selection.set(Some(path));
                        selecting_directory.set(None);
                    }
                    _ => {
                        selection.set(Some(path.clone()));
                        selecting_directory.set(Some(path));
                    }
                }
            } else if !directory && !with_directory {
                selection.set(Some(path));
                selecting_directory.set(None);
            } else if !directory && with_directory {
                selection.set(None);
                selecting_directory.set(Some(path));
            }
        }
    };

    use_future(move || async move {
        status.set(DirectoryInfoState::Fetching);

        match get_directory_info(current_path.cloned()).await {
            Ok(dir_info) => status.set(DirectoryInfoState::Completed(dir_info)),
            Err(err) => status.set(DirectoryInfoState::Error(Rc::new(err))),
        }
    });

    rsx! {
        div {
            class: "file-select",

            div {
                class: "file-select-header",

                div {
                    class: "file-select-header-title",

                    span {
                        "Select a "
                        if !directory {
                            "file"
                        } else {
                            "directory"
                        }
                    }
                }

                div {
                    class: "file-select-header-path",

                    PathDisplay {
                        path: current_path()
                    }

                    span {
                        dangerous_inner_html: "&#8203;"
                    }
                }

                div {
                    class: "file-select-header-actions",

                    button {
                        r#type: "button",
                        class: "icon-button",
                        disabled: current_path() == PathBuf::from("") || current_path() == PathBuf::from("/"),
                        onclick: move |_| {
                            if let Some(path) = current_path().parent() {
                                current_path.set(path.to_path_buf());
                                selection.set(Some(path.to_path_buf()));
                            } else {
                                current_path.set(PathBuf::from(""));
                                selection.set(None);
                            }
                        },

                        Icon {
                            data: ARROW_UP
                        }
                    }
                }
            }

            div {
                class: "file-select-body",

                match status() {
                    DirectoryInfoState::Fetching => rsx! {
                        Loading {
                            class: "dir-info-fetching",
                            text: "Fetching directory info..."
                        }
                    },
                    DirectoryInfoState::Completed(dir_info) => if !dir_info.dirs.is_empty() || !dir_info.files.is_empty() {
                        let dirs = dir_info.dirs.into_iter().map(|entry| {
                            let mut classes = vec!["dir-info-dir"];

                            if let Some(path) = selection() {
                                if path.to_str() == Some(entry.as_str()) {
                                    classes.push("dir-info-selected");
                                }
                            }

                            if let Some(path) = selecting_directory() {
                                if path.to_str() == Some(entry.as_str()) {
                                    classes.push("dir-info-selecting-directory");
                                }
                            }

                            let classes = classes.join(" ");
                            let entry_str = entry.strip_suffix(MAIN_SEPARATOR).unwrap_or(entry.as_str());

                            rsx! {
                                div {
                                    class: "{classes}",
                                    onclick: move |_| set_selection(entry.as_str(), true),

                                    Icon {
                                        data: FOLDER,
                                        class: "folder-icon"
                                    }
                                    span {
                                        "{entry_str}"
                                    }
                                }
                            }
                        });

                        let files = dir_info.files.into_iter().map(|entry| {
                            let mut classes = vec!["dir-info-file"];

                            if let Some(path) = selection() {
                                if path.to_str() == Some(entry.as_str()) {
                                    classes.push("dir-info-selected");
                                }
                            }

                            let classes = classes.join(" ");

                            rsx! {
                                div {
                                    class: "{classes}",
                                    onclick: move |_| set_selection(entry.as_str(), false),

                                    Icon {
                                        data: FILE,
                                        class: "file-icon"
                                    }
                                    span {
                                        "{entry}"
                                    }
                                }
                            }
                        });

                        rsx! {
                            div {
                                class: "dir-info",

                                {dirs}
                                {files}
                            }
                        }
                    } else {
                        rsx! {
                            div {
                                class: "dir-info-info",
                                "Empty directory"
                            }
                        }
                    },
                    DirectoryInfoState::Error(err) => rsx! {
                        div {
                            class: "error dir-info-error",
                            "An error occurred while fetching directory info: "
                            {err.to_string()}
                        }
                    },
                }
            }

            div {
                class: "file-select-footer",

                div {
                    class: "file-select-selection-container",

                    match selection() {
                        Some(path) => {
                            match path.iter()
                                .filter_map(|component|
                                    component.to_str()
                                        .filter(|&component| component != MAIN_SEPARATOR_STR))
                                .last()
                            {
                                Some(name) => rsx! {
                                    div {
                                        class: "file-select-selection",
                                        "Selected: "
                                        "{name}"
                                    }
                                },
                                None => rsx! {
                                    div {
                                        class: "file-select-selection",
                                        "No path selected"
                                    }
                                }
                            }
                        },
                        None => rsx! {
                            div {
                                class: "file-select-selection",
                                "No path selected"
                            }
                        },
                    }
                }

                div {
                    class: "file-select-actions-container",

                    div {
                        class: "file-select-actions",

                        if cancelable {
                            button {
                                r#type: "button",
                                class: "button secondary",
                                onclick: move |_| if let Some(on_cancel) = &on_cancel {
                                    on_cancel.call(());
                                },
                                "Cancel"
                            }
                        }

                        button {
                            r#type: "button",
                            class: "button primary",
                            disabled: selection().is_none(),
                            onclick: move |_| on_select.call(current_path().join(selection().unwrap())),
                            "Select"
                        }
                    }
                }
            }
        }
    }
}
