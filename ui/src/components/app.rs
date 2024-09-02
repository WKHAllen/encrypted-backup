//! Root-level application component.

use dioxus::prelude::*;

/// The root application component.
#[component]
pub fn App() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div { "Count: {count}" }
        button { onclick: move |_| count += 1, "+" }
        button { onclick: move |_| count -= 1, "-" }
    }
}
