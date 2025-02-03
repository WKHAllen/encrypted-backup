//! Dialog UI component.

use super::{Button, ButtonStyle, IconButton};
use crate::classes::*;
use crate::icons::XMARK;
use dioxus::prelude::*;
use macros::*;

/// Dialog size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, ClassName)]
pub enum DialogSize {
    /// A small dialog.
    Small,
    /// A medium dialog.
    #[default]
    Medium,
    /// A large dialog.
    Large,
    /// A maximum size dialog.
    Max,
    /// An automatically sized dialog.
    Auto,
}

/// Dialog action buttons layout.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, ClassName)]
pub enum DialogActionsLayout {
    /// Left-aligned actions.
    Left,
    /// Right-aligned actions.
    #[default]
    Right,
    /// Actions spaced across the line.
    Spaced,
}

/// A dialog UI component.
#[component]
pub fn Dialog(
    /// The dialog state.
    state: Signal<bool>,
    /// The dialog size.
    #[props(default)]
    size: DialogSize,
    /// The dialog title.
    title: String,
    /// The ok button label. Will not be created if empty.
    ok_label: Option<String>,
    /// The cancel button label. Will not be created if empty.
    cancel_label: Option<String>,
    /// The callback called with the dialog closing state. Receives `true` if
    /// the ok button was clicked and `false` otherwise.
    #[props(default)]
    on_close_request: EventHandler<bool>,
    /// Whether to close the dialog when the ok button is clicked.
    #[props(default = true)]
    close_on_ok: bool,
    /// Whether to close the dialog when the cancel button is clicked.
    #[props(default = true)]
    close_on_cancel: bool,
    /// The layout of action buttons.
    #[props(default)]
    actions_layout: DialogActionsLayout,
    /// Elements within the dialog.
    children: Element,
) -> Element {
    let size_class = format!("dialog-{}", size.class_name());
    let actions_layout_class = format!("dialog-actions-{}", actions_layout.class_name());

    let mut mouse_in = use_signal(|| false);

    rsx! {
        div {
            class: classes!("dialog-container", state().then_some("dialog-container-open")),
            onclick: move |_| {
                if !mouse_in() {
                    on_close_request.call(false);
                    state.set(false);
                }
            },

            div {
                class: classes!("dialog", size_class),
                onmouseenter: move |_| {
                    mouse_in.set(true);
                },
                onmouseleave: move |_| {
                    mouse_in.set(false);
                },

                div {
                    class: "dialog-inner",

                    div {
                        class: "dialog-header",

                        div {
                            class: "dialog-header-space",

                            h3 {
                                class: "dialog-title",
                                "{title}"
                            }

                            IconButton {
                                data: XMARK,
                                onclick: move |_| {
                                    on_close_request.call(false);
                                    state.set(false);
                                }
                            }
                        }
                    }

                    div {
                        class: "dialog-body",
                        {children}
                    }

                    div {
                        class: classes!("dialog-actions", actions_layout_class),

                        if let Some(label) = cancel_label {
                            Button {
                                text: "{label}",
                                style: ButtonStyle::Transparent,
                                onclick: move |_| {
                                    on_close_request.call(false);

                                    if close_on_cancel {
                                        state.set(false);
                                    }
                                }
                            }
                        }

                        if let Some(label) = ok_label {
                            Button {
                                text: "{label}",
                                style: ButtonStyle::Primary,
                                onclick: move |_| {
                                    on_close_request.call(true);

                                    if close_on_ok {
                                        state.set(false);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
