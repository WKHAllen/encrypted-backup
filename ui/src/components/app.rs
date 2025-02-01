//! Root-level application component.

use super::{ControlError, Loading, RunningOperation};
use crate::components::Config;
use crate::logger::*;
use crate::services::{Config as ConfigState, Operation};
use dioxus::prelude::*;
use std::io;
use std::rc::Rc;
use tokio::sync::mpsc::unbounded_channel;

/// The global stylesheet asset.
const STYLES: &str = include_str!("../../assets/css/main.css");

/// The initial application loading state.
#[derive(Debug, Clone)]
enum AppState {
    /// The application is loading.
    Loading,
    /// The application failed while loading.
    FailedLoading(Rc<io::Error>),
    /// The application has loaded successfully and the configuration menu is
    /// active.
    Configuring(ConfigState),
    /// An operation is running.
    Running(Operation),
}

/// The root application component.
#[component]
pub fn App() -> Element {
    let mut app_state = use_signal(|| AppState::Loading);

    let mut get_config = use_future(move || async move {
        match ConfigState::load().await {
            Ok(config) => app_state.set(AppState::Configuring(config)),
            Err(err) => app_state.set(AppState::FailedLoading(Rc::new(err))),
        }
    });

    let logging_receiver = use_signal(|| {
        let (sender, receiver) = unbounded_channel();
        init_logger(true, sender).unwrap();
        receiver
    });

    rsx! {
        div {
            class: "app",

            style {
                "{STYLES}"
            }

            match app_state() {
                AppState::Loading => rsx! {
                    Loading {
                        class: "app-loading",
                        text: "Loading...",
                    }
                },
                AppState::FailedLoading(err) => rsx! {
                    ControlError {
                        message: Some(err.to_string()),
                    }
                },
                AppState::Configuring(config) => rsx! {
                    Config {
                        config: config,
                        start: move |operation| {
                            app_state.set(AppState::Running(operation));
                        },
                    }
                },
                AppState::Running(operation) => rsx! {
                    RunningOperation {
                        operation: operation,
                        logging_receiver: logging_receiver,
                        back: move |_| {
                            app_state.set(AppState::Loading);
                            get_config.restart();
                        },
                    }
                },
            }
        }
    }
}
