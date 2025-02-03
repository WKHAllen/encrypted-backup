//! UI component for running a backup/extraction operation.

use crate::services::Operation;
use dioxus::prelude::*;
use std::thread;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

/// A component that performs a backup/extraction operation and displays its
/// progress.
#[component]
pub fn RunningOperation(
    /// The operation configuration.
    operation: Operation,
    /// The logging receiver.
    logging_receiver: Signal<UnboundedReceiver<String>>,
    /// The callback to trigger going back to the configuration menu.
    back: EventHandler<()>,
) -> Element {
    let mut logs = use_signal(Vec::new);

    let (cancel_sender, mut cancel_receiver) = use_hook(|| {
        let (tx, rx) = unbounded_channel();
        (Signal::new(tx), Signal::new(rx))
    });

    use_future(move || async move {
        loop {
            let mut logging_ref = logging_receiver.write();
            let mut cancel_ref = cancel_receiver.write();

            select! {
                maybe_line = logging_ref.recv() => {
                    match maybe_line {
                        Some(line) => logs.write().push(line),
                        None => {
                            break;
                        },
                    }
                },
                _ = cancel_ref.recv() => {
                    break;
                },
            }
        }
    });

    use_drop(move || {
        let _ = cancel_sender.read().send(());
    });

    let mut res = use_signal_sync(|| None);

    let _operation_handle = use_signal({
        let operation = operation.clone();
        move || {
            thread::spawn(move || {
                let operation_res = operation.execute();
                res.set(Some(operation_res));
            })
        }
    });

    rsx! {
        div {
            class: "running-operation",

            h2 {
                class: "running-operation-title",

                if operation.is_backup() {
                    "Performing backup"
                } else {
                    "Performing extraction"
                }
            }

            div {
                class: "running-operation-logs",

                for line in logs() {
                    span {
                        "{line}"
                    }
                }
            }

            div {

            }
        }
    }
}
