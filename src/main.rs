mod backup;
mod backup_crypto;
mod crypto;
mod logger;
mod types;

use crate::types::*;
use clap::{Parser, Subcommand};
use glob::Pattern;
use std::path::PathBuf;

/// A tool to securely back up files and directories.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

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
        /// extracted.
        #[arg(short, long, value_parser = validate_password)]
        password: Option<String>,
        /// Size of each chunk of the backup, as an order of magnitude. For a
        /// provided chunk size magnitude n, each chunk will be 2^n bytes. A
        /// higher chunk size means a faster backup, but greater memory usage.
        /// The default magnitude is 16, equivalent to a chunk size of 64 KiB.
        /// Note that the same chunk size will be used to extract the backup.
        #[arg(short, long, value_parser = validate_chunk_size, default_value_t = 16)]
        chunk_size_magnitude: u8,
        /// Asynchronous file I/O mode. Disabled by default. Enabling this
        /// generally makes things slower.
        #[arg(short, long, value_parser, default_value_t = false)]
        async_io: bool,
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
        /// Password for the backup file.
        #[arg(short, long, value_parser)]
        password: Option<String>,
        /// Asynchronous file I/O mode. Disabled by default. Enabling this
        /// generally makes things slower.
        #[arg(short, long, value_parser, default_value_t = false)]
        async_io: bool,
        /// Debug mode.
        #[arg(short, long, value_parser, default_value_t = false)]
        debug: bool,
    },
}

fn validate_file(path_str: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path_str);

    if !path.exists() {
        Err(format!("Path does not exist: {}", path_str))
    } else if !path.is_file() {
        Err(format!("Path is not a file: {}", path_str))
    } else {
        Ok(path)
    }
}

fn validate_path(path_str: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path_str);

    if !path.exists() {
        Err(format!("Path does not exist: {}", path.display()))
    } else if !path.is_dir() && !path.is_file() {
        Err(format!(
            "Path is not a file or directory: {}",
            path.display()
        ))
    } else {
        Ok(path)
    }
}

fn validate_glob(glob_str: &str) -> Result<Pattern, String> {
    let glob_result = Pattern::new(glob_str);

    match glob_result {
        Ok(glob_pattern) => Ok(glob_pattern),
        Err(_e) => Err(format!("Invalid glob: {}", glob_str)),
    }
}

fn validate_password(password: &str) -> Result<String, String> {
    if password.len() < 8 {
        Err("Password must be at least 8 characters in length".to_owned())
    } else if password.len() > 255 {
        Err("Password must be at most 255 characters in length".to_owned())
    } else {
        Ok(password.to_owned())
    }
}

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

fn get_password(password: Option<String>, confirm: bool, validate: bool) -> Result<String, String> {
    match password {
        Some(pw) => Ok(pw),
        None => {
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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup {
            include_paths,
            exclude_globs,
            output_path,
            password,
            chunk_size_magnitude,
            async_io,
            debug,
        } => {
            logger::init(debug).unwrap();

            // Fail immediately if not enough memory can be allocated
            let chunk = vec![0u8; 1 << chunk_size_magnitude];
            drop(chunk);

            match get_password(password, true, true) {
                Ok(pw) => match backup::backup(
                    &include_paths,
                    &exclude_globs,
                    &output_path,
                    &pw,
                    1 << chunk_size_magnitude,
                    async_io,
                ) {
                    Ok(path) => println!("Successfully backed up to {}", path.display()),
                    Err(e) => println!("Failed to perform backup: {}", e),
                },
                Err(e) => println!("Invalid password: {}", e),
            }
        }
        Commands::Extract {
            backup_path,
            output_path,
            password,
            async_io,
            debug,
        } => {
            logger::init(debug).unwrap();

            match get_password(password, false, false) {
                Ok(pw) => match backup::extract(&backup_path, &output_path, &pw, async_io) {
                    Ok(path) => println!("Successfully extracted to {}", path.display()),
                    Err(e) => {
                        println!("Failed to perform extraction: {}", e);

                        match e {
                            BackupError::CryptoError(_) => println!("This usually means that the provided password was incorrect, and cannot be used to extract the backup."),
                            _ => (),
                        }
                    }
                },
                Err(e) => println!("Invalid password: {}", e),
            }
        }
    }
}
