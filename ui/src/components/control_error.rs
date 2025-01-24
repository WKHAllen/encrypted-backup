//! An error on a controlled input.

use dioxus::prelude::*;

/// An optional error on a controlled input.
#[component]
pub fn ControlError(
    /// The optional error message. If empty, the error will not be displayed.
    #[props(!optional, default)]
    message: Option<String>,
) -> Element {
    let message_text = message.unwrap_or_default();

    rsx! {
        span {
            class: "control-error",
            "{message_text}"
        }
    }
}
