//! Button UI component.

use crate::classes::*;
use dioxus::prelude::*;
use macros::*;

/// The style of a button.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, ClassName)]
pub enum ButtonStyle {
    /// Primary style.
    #[default]
    Primary,
    /// Transparent style.
    Transparent,
}

/// A button UI component.
#[component]
pub fn Button(
    /// The text on the button.
    text: String,
    /// The button style.
    #[props(default)]
    style: ButtonStyle,
    /// Whether the button is disabled.
    #[props(default = false)]
    disabled: bool,
    /// An optional class name for the icon button.
    class: Option<String>,
    /// The button click callback.
    #[props(default)]
    onclick: EventHandler<()>,
) -> Element {
    let style_class = format!("button-{}", style.class_name());

    rsx! {
        button {
            r#type: "button",
            class: classes!("button", style_class, class),
            onclick: move |_| {
                onclick.call(());
            },
            disabled: disabled,

            "{text}"
        }
    }
}
