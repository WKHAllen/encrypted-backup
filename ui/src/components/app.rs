//! Root-level application component.

use super::{ControlError, Loading};
use crate::components::Config;
use crate::services::Config as ConfigState;
use dioxus::prelude::*;
use std::io;
use std::rc::Rc;

/// The global stylesheet asset.
const STYLES: &str = include_str!("../../assets/css/main.css");

/// The initial application loading state.
#[derive(Debug, Clone)]
enum AppLoadingState {
    /// The application is loading.
    Loading,
    /// The application has loaded successfully.
    Completed(ConfigState),
    /// The application failed while loading.
    Failed(Rc<io::Error>),
}

/// The root application component.
#[component]
pub fn App() -> Element {
    let mut loading_state = use_signal(|| AppLoadingState::Loading);

    use_future(move || async move {
        match ConfigState::load().await {
            Ok(config) => loading_state.set(AppLoadingState::Completed(config)),
            Err(err) => loading_state.set(AppLoadingState::Failed(Rc::new(err))),
        }
    });

    rsx! {
        div {
            class: "app",

            style {
                "{STYLES}"
            }

            match loading_state() {
                AppLoadingState::Loading => rsx! {
                    Loading {
                        class: "app-loading",
                        text: "Loading...",
                    }
                },
                AppLoadingState::Completed(config) => rsx! {
                    Config {
                        config: config,
                    }
                },
                AppLoadingState::Failed(err) => rsx! {
                    ControlError {
                        message: Some(err.to_string()),
                    }
                }
            }
        }
    }
}
