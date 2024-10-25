//! A tool to securely back up files and directories.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(unused_mut)]
#![warn(clippy::missing_docs_in_private_items)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::if_not_else)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::future_not_send)]
#![allow(non_snake_case)]
// TODO: remove later
#![allow(dead_code)]
// TODO: remove later
#![allow(unused_imports)]

mod classes;
mod components;
mod icons;
mod services;

use crate::components::App;
use dioxus::prelude::*;

fn main() {
    launch_desktop(App);
}
