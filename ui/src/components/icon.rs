//! An icon component.

use crate::classes::*;
use dioxus::prelude::*;

/// Icon component.
#[component]
pub fn Icon(
    /// The raw icon data.
    data: String,
    /// An optional class name for the icon element.
    class: Option<String>,
) -> Element {
    let class = classes!("icon", class);

    rsx! {
        div {
            class: "{class}",
            dangerous_inner_html: data
        }
    }
}
