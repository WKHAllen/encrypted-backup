//! Path displaying component.

use dioxus::prelude::*;
use std::path::{PathBuf, MAIN_SEPARATOR_STR};

/// Path component.
#[component]
pub fn PathDisplay(
    /// The path to display.
    #[props(into)]
    path: PathBuf,
) -> Element {
    let components = path.iter().filter_map(|component| {
        component
            .to_str()
            .filter(|&component| component != MAIN_SEPARATOR_STR)
    });
    let num_components = components.clone().count();
    let components = components
        .flat_map(|component| {
            [
                rsx! {
                    div {
                        class: "path-component",

                        span {
                            {component}
                        }
                    }
                },
                rsx! {
                    div {
                        class: "path-separator",

                        span {
                            "/"
                        }
                    }
                },
            ]
        })
        .take((num_components * 2).saturating_sub(1));

    rsx! {
        div {
            class: "path",

            {components}
        }
    }
}
