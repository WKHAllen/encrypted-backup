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

mod backup;
mod backup_crypto;
mod crypto;
mod logger;
mod memory;
mod pool;
mod types;
mod util;

pub use crate::backup::{backup, backup_chunk_size, extract};
pub use crate::logger::init_logger;
pub use crate::memory::check_memory;
pub use crate::types::{BackupError, BackupResult};
