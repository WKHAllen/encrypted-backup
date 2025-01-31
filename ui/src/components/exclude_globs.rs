//! UI component for glob exclusion.

use super::{ControlError, IconButton};
use crate::classes::*;
use crate::icons::{ARROW_DOWN, ARROW_UP, PLUS, XMARK};
use crate::services::parse_pattern;
use dioxus::prelude::*;
use glob::{Pattern, PatternError};
use std::rc::Rc;

/// UI component to specify glob exclusions for the backup.
#[component]
pub fn ExcludeGlobs(
    /// The glob exclusions state.
    #[allow(clippy::type_complexity)]
    state: Signal<Vec<Result<Pattern, (String, Rc<PatternError>)>>>,
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

    let first_error_message = state.read().iter().find_map(|pattern| match pattern {
        Ok(_) => None,
        Err((_, err)) => Some(err.msg.to_owned()),
    });

    rsx! {
        div {
            class: "exclude-globs-container",

            span {
                "Exclude globs"
            }

            div {
                class: "exclude-globs",

                div {
                    class: "exclude-globs-actions",

                    IconButton {
                        data: PLUS,
                        onclick: move |_| {
                            state.with_mut(|patterns| patterns.push(parse_pattern("")));
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
                    class: "exclude-globs-globs",

                    if !state.with(Vec::is_empty) {
                        for (index, pattern) in state.read().iter().enumerate() {
                            div {
                                class: classes!("exclude-globs-glob", (selected_index() == Some(index)).then_some("exclude-globs-glob-selected")),

                                {
                                    let pattern_str = match pattern {
                                        Ok(pat) => pat.as_str(),
                                        Err((invalid_pat, _)) => invalid_pat.as_str(),
                                    };

                                    rsx! {
                                        input {
                                            class: classes!("exclude-globs-glob-input", pattern.is_err().then_some("exclude-globs-glob-input-invalid")),
                                            r#type: "text",
                                            value: "{pattern_str}",
                                            onclick: move |_| {
                                                selected_index.set(Some(index));
                                            },
                                            oninput: move |event| {
                                                state.with_mut(|patterns| patterns[index] = parse_pattern(event.value()));
                                            },
                                            {pattern_str}
                                        }
                                    }
                                }

                                IconButton {
                                    data: XMARK,
                                    class: "exclude-globs-glob-remove",
                                    onclick: move |_| {
                                        let _ = state.with_mut(|patterns| patterns.remove(index));

                                        if let Some(selected) = selected_index() {
                                            if selected == index {
                                                selected_index.set(None);
                                            } else if selected > index {
                                                selected_index.set(Some(selected - 1));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            class: "exclude-globs-globs-empty",

                            span {
                                "No exclusion globs specified"
                            }
                        }
                    }
                }
            }

            span {
                class: "exclude-globs-info",
                "Paths matching these glob patterns will be excluded from the backup"
            }

            ControlError {
                message: first_error_message.map(|err| format!("Glob pattern parse error: {err}")),
            }
        }
    }
}
