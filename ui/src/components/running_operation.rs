//! UI component for running a backup/extraction operation.

use crate::services::Operation;
use dioxus::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;

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
    rsx! {
        // TODO
        span {
            "Running operation"
        }
    }
}
