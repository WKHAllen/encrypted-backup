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

mod logger;

use crate::logger::*;
use backup::*;
use clap::{Parser, Subcommand};
use glob::Pattern;
use std::path::{Path, PathBuf};
use std::process::exit;

/// A tool to securely back up files and directories.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Cli {
    /// Encrypted backup subcommands.
    #[command(subcommand)]
    command: Commands,
}

/// Encrypted backup subcommands.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Backs up and encrypts files and directories.
    Backup {
        /// Paths to include in the backup.
        #[arg(required = true, value_parser = validate_path)]
        include_paths: Vec<PathBuf>,
        /// Globs to exclude from the backup, separated by commas.
        #[arg(short, long, value_delimiter = ',', value_parser = validate_glob)]
        exclude_globs: Vec<Pattern>,
        /// Output path of the backup.
        #[arg(short, long, required = true, value_parser = validate_output_path)]
        output_path: PathBuf,
        /// Password for the backup file. The same password will be needed to
        /// extract the backup later. Without it, the backup cannot be
        /// extracted. If not provided, the password will be prompted from
        /// standard input.
        #[arg(short, long, value_parser = validate_password)]
        password: Option<String>,
        /// Size of each chunk of the backup, as an order of magnitude. For a
        /// provided chunk size magnitude n, each chunk will be 2^n bytes. A
        /// higher chunk size means a faster backup, but greater memory usage.
        /// The default magnitude is 16, equivalent to a chunk size of 64 KiB.
        /// Note that the same chunk size will be used to extract the backup.
        #[arg(short, long, value_parser = validate_chunk_size, default_value_t = 16)]
        chunk_size_magnitude: u8,
        /// Number of workers to spawn in the pool that will perform crypto
        /// operations in parallel. The default pool size is 4. The optimal size
        /// is typically closer to 16, but higher numbers will be more taxing on
        /// the CPU.
        #[arg(long, value_parser = validate_pool_size, default_value_t = 4)]
        pool_size: u8,
        /// Overrides the 1GB memory limit.
        #[arg(long, value_parser, default_value_t = false)]
        override_memory_limit: bool,
        /// Debug mode.
        #[arg(short, long, value_parser, default_value_t = false)]
        debug: bool,
    },
    /// Decrypts and extracts an encrypted backup.
    Extract {
        /// Path to the encrypted backup.
        #[arg(required = true, value_parser = validate_file)]
        backup_path: PathBuf,
        /// Path to extract the backup to.
        #[arg(short, long, value_parser = validate_output_path)]
        output_path: PathBuf,
        /// Password for the backup file. If not provided, the password will
        /// be prompted from standard input.
        #[arg(short, long, value_parser)]
        password: Option<String>,
        /// Number of workers to spawn in the pool that will perform crypto
        /// operations in parallel. The default pool size is 16. This is
        /// usually an optimal size, and can speed things up substantially.
        #[arg(short, long, value_parser = validate_pool_size, default_value_t = 16)]
        pool_size: u8,
        /// Overrides the 1GB memory limit.
        #[arg(long, value_parser, default_value_t = false)]
        override_memory_limit: bool,
        /// Debug mode.
        #[arg(short, long, value_parser, default_value_t = false)]
        debug: bool,
    },
}

/// Validates that a provided path exists and is a file.
fn validate_file(path_str: &str) -> Result<PathBuf, String> {
    let path = Path::new(path_str);

    if !path.exists() {
        Err(format!("Path does not exist: {path_str}"))
    } else if !path.is_file() {
        Err(format!("Path is not a file: {path_str}"))
    } else {
        Ok(path.to_owned())
    }
}

/// Validates that a provided path exists and is either a file or directory.
fn validate_path(path_str: &str) -> Result<PathBuf, String> {
    let path = Path::new(path_str);

    if !path.exists() {
        Err(format!("Path does not exist: {}", path.display()))
    } else if !path.is_dir() && !path.is_file() {
        Err(format!(
            "Path is not a file or directory: {}",
            path.display()
        ))
    } else {
        Ok(path.to_owned())
    }
}

/// Validates that a glob is legitimate.
fn validate_glob(glob_str: &str) -> Result<Pattern, String> {
    Pattern::new(glob_str).map_err(|e| format!("Invalid glob: {glob_str}, {e}"))
}

/// Validates that a password is of the correct length.
fn validate_password(password: &str) -> Result<String, String> {
    if password.len() < 8 {
        Err("Password must be at least 8 characters in length".to_owned())
    } else if password.len() > 255 {
        Err("Password must be at most 255 characters in length".to_owned())
    } else {
        Ok(password.to_owned())
    }
}

/// Validates that a provided output path does not yet exist and has a valid
/// parent directory.
fn validate_output_path(path_str: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path_str);

    match path.parent() {
        Some(parent) => {
            if parent.exists() {
                if !path.exists() {
                    Ok(path)
                } else {
                    Err(format!("Path already exists: {}", path.display()))
                }
            } else {
                Err(format!("Parent path does not exist: {}", path.display()))
            }
        }
        None => Err(format!("Could not get parent path: {}", path.display())),
    }
}

/// Validates that the provided chunk size is within the accepted range.
fn validate_chunk_size(chunk_size: &str) -> Result<u8, String> {
    let size = chunk_size.parse::<u8>().map_err(|e| e.to_string())?;

    if size < 10 {
        Err("Chunk size order of magnitude must be at least 10".to_owned())
    } else if size > 30 {
        Err("Chunk size order of magnitude must be at most 30".to_owned())
    } else {
        Ok(size)
    }
}

/// Validates that the provided pool size is within the accepted range.
fn validate_pool_size(pool_size: &str) -> Result<u8, String> {
    let size = pool_size.parse::<u8>().map_err(|e| e.to_string())?;

    if size < 1 {
        Err("Pool size must be at least 1".to_owned())
    } else if size > 64 {
        Err("Pool size must be at most 64".to_owned())
    } else {
        Ok(size)
    }
}

/// Prompts for the password from standard input.
fn get_password(password: Option<String>, confirm: bool, validate: bool) -> Result<String, String> {
    if let Some(pw) = password {
        Ok(pw)
    } else {
        let pw = rpassword::prompt_password("Backup password: ").unwrap();

        if confirm {
            let pw_confirm = rpassword::prompt_password("Confirm password: ").unwrap();

            if pw != pw_confirm {
                return Err("Passwords do not match".to_owned());
            }
        }

        if validate {
            validate_password(&pw)
        } else {
            Ok(pw)
        }
    }
}

/// Attempt to perform a backup or extraction.
fn perform_backup(command: Commands) -> Result<String, String> {
    match command {
        Commands::Backup {
            include_paths,
            exclude_globs,
            output_path,
            password,
            chunk_size_magnitude,
            pool_size,
            override_memory_limit,
            debug,
        } => {
            init_logger(debug).unwrap();

            let chunk_size = 1 << chunk_size_magnitude;
            check_memory(chunk_size, pool_size, override_memory_limit)?;

            match get_password(password, true, true) {
                Ok(pw) => match backup::backup(
                    &include_paths,
                    &exclude_globs,
                    output_path,
                    &pw,
                    chunk_size,
                    pool_size,
                ) {
                    Ok(path) => Ok(format!("Successfully backed up to {}", path.display())),
                    Err(e) => Err(format!("Failed to perform backup: {e}")),
                },
                Err(e) => Err(format!("Invalid password: {e}")),
            }
        }
        Commands::Extract {
            backup_path,
            output_path,
            password,
            pool_size,
            override_memory_limit,
            debug,
        } => {
            init_logger(debug).unwrap();

            let chunk_size = backup::backup_chunk_size(&backup_path)
                .map_err(|e| format!("Failed to perform extraction: {e}"))?;
            check_memory(chunk_size, pool_size, override_memory_limit)?;

            match get_password(password, false, false) {
                Ok(pw) => match backup::extract(backup_path, output_path, &pw, pool_size) {
                    Ok(path) => Ok(format!("Successfully extracted to {}", path.display())),
                    Err(e) => Err(if let BackupError::CryptoError(_) = e {
                        format!("Failed to perform extraction: {e}.\nThis usually means that the provided password was incorrect, and cannot be used to extract the backup.")
                    } else {
                        format!("Failed to perform extraction: {e}")
                    }),
                },
                Err(e) => Err(format!("Invalid password: {e}")),
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match perform_backup(cli.command) {
        Ok(msg) => println!("{msg}"),
        Err(msg) => {
            eprintln!("{msg}");
            exit(1);
        }
    }
}
