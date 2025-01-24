//! Element ID hook.

use dioxus::prelude::*;

/// Generates a new random ID for an element.
fn new_id() -> String {
    format!("{:x}", rand::random::<u32>())
}

/// Generates a new random ID for an element and wraps it in a signal so that it
/// can remain the same across renders.
pub fn use_id() -> ReadOnlySignal<String> {
    use_signal(new_id).into()
}
