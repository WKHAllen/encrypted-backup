//! An icon button component.

use super::Icon;
use crate::classes::*;
use dioxus::prelude::*;

/// Icon button component.
#[component]
pub fn IconButton(
    /// The raw icon data.
    data: String,
    /// Whether the button is disabled.
    #[props(default = false)]
    disabled: bool,
    /// An optional class name for the icon button.
    class: Option<String>,
    /// The on click handler.
    onclick: EventHandler<()>,
) -> Element {
    let class = classes!(
        "icon-button",
        disabled.then_some("icon-button-disabled"),
        class
    );

    rsx! {
        div {
            class: class,
            onclick: move |_| if !disabled {
                onclick.call(());
            },

            Icon {
                data: data,
            }
        }
    }
}
