//! Root-level application component.

use crate::components::Config;
use dioxus::prelude::*;

/// The global stylesheet asset.
const STYLES: &str = include_str!("../../assets/css/main.css");

/// The root application component.
#[component]
pub fn App() -> Element {
    rsx! {
        div {
            class: "app",

            style {
                "{STYLES}"
            }

            Config {}
        }
    }
}
