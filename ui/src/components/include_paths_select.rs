//! UI component for include path selection.

use super::{FileDialog, IconButton};
use crate::classes::*;
use crate::icons::{ARROW_DOWN, ARROW_UP, PLUS, XMARK};
use dioxus::prelude::*;
use std::path::PathBuf;

/// UI component to select paths to include in a backup.
#[component]
pub fn IncludePathsSelect(
    /// The include paths state.
    state: Signal<Vec<PathBuf>>,
) -> Element {
    let mut selected_index = use_signal(|| None);

    let can_move_up = match selected_index() {
        Some(index) => index > 0,
        None => false,
    };
    let can_move_down = match selected_index() {
        Some(index) => index < state.with(Vec::len) - 1,
        None => false,
    };

    rsx! {
        div {
            class: "include-paths-select-container",

            span {
                "Include paths"
            }

            div {
                class: "include-paths-select",

                div {
                    class: "include-paths-select-actions",

                    FileDialog {
                        directory: true,
                        onselect: move |path| {
                            if let Some(path) = path {
                                state.with_mut(|paths| paths.push(path));
                            }
                        },

                        IconButton {
                            data: PLUS,
                            onclick: move |_| {}
                        }
                    }

                    IconButton {
                        data: ARROW_UP,
                        disabled: !can_move_up,
                        onclick: move |_| {
                            if let Some(index) = selected_index() {
                                state.with_mut(|paths| paths.swap(index, index - 1));
                                selected_index.set(Some(index - 1));
                            }
                        }
                    }

                    IconButton {
                        data: ARROW_DOWN,
                        disabled: !can_move_down,
                        class: "rotate-180",
                        onclick: move |_| {
                            if let Some(index) = selected_index() {
                                state.with_mut(|paths| paths.swap(index, index + 1));
                                selected_index.set(Some(index + 1));
                            }
                        }
                    }
                }

                div {
                    class: "include-paths-select-paths",

                    if !state.with(Vec::is_empty) {
                        for (index, path) in state.read().iter().enumerate() {
                            div {
                                class: classes!("include-paths-select-path", (selected_index() == Some(index)).then_some("include-paths-select-path-selected")),

                                div {
                                    class: "include-paths-select-path-label",
                                    onclick: move |_| {
                                        selected_index.set(Some(index));
                                    },
                                    {path.display().to_string()}
                                }

                                IconButton {
                                    data: XMARK,
                                    class: "include-paths-select-path-remove",
                                    onclick: move |_| {
                                        state.with_mut(|paths| paths.remove(index));
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            class: "include-paths-select-paths-empty",

                            span {
                                "No include paths selected"
                            }
                        }
                    }
                }
            }
        }
    }
}
