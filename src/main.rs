mod backup;
mod crypto;
mod logger;

use clap::{Parser, Subcommand};
use glob::Pattern;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Backs up and encrypts files and directories
    Backup {
        /// Comma-separated paths to include in the backup
        #[clap(required = true, multiple_values = true, value_delimiter = ',', value_parser = validate_path)]
        include_paths: Vec<PathBuf>,
        /// Comma-separated globs to exclude from the backup
        #[clap(short, long, multiple_values = true, value_delimiter = ',', value_parser = validate_glob)]
        exclude_globs: Vec<Pattern>,
        /// Directory to save the backup to
        #[clap(short, long, required = true, value_parser = validate_dir)]
        output_dir: PathBuf,
        /// Name of the backup file
        #[clap(short, long, required = true, value_parser)]
        name: String,
        /// Password for the backup file
        #[clap(short, long, required = true, value_parser = validate_password)]
        password: String,
        /// Debug mode
        #[clap(short, long, value_parser, default_value_t = false)]
        debug: bool,
    },
    /// Decrypts and extracts an encrypted backup
    Extract {
        /// Path to the encrypted backup
        #[clap(required = true, value_parser = validate_file)]
        backup_path: PathBuf,
        /// Path to extract the backup into
        #[clap(short, long, value_parser = validate_extract_output_path)]
        output_path: Option<PathBuf>,
        /// Password for the backup file
        #[clap(short, long, required = true, value_parser = validate_password)]
        password: String,
        /// Debug mode
        #[clap(short, long, value_parser, default_value_t = false)]
        debug: bool,
    },
}

/// A tool to securely back up files and directories
#[derive(Parser, Debug)]
#[clap(about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
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

fn validate_dir(path_str: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path_str);

    if !path.exists() {
        Err(format!("Path does not exist: {}", path_str))
    } else if !path.is_dir() {
        Err(format!("Path is not a directory: {}", path_str))
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

fn validate_extract_output_path(path_str: &str) -> Result<PathBuf, String> {
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup {
            include_paths,
            exclude_globs,
            output_dir,
            name,
            password,
            debug,
        } => {
            logger::init(debug).unwrap();

            match backup::backup(
                &include_paths,
                &exclude_globs,
                &output_dir,
                &name,
                &password,
            ) {
                Ok(path) => println!("Successfully backed up to {}", path.display()),
                Err(e) => println!("Failed to perform backup: {}", e),
            }
        }
        Commands::Extract {
            backup_path,
            output_path,
            password,
            debug,
        } => {
            logger::init(debug).unwrap();

            match backup::extract(&backup_path, output_path, &password) {
                Ok(path) => println!("Successfully extracted to {}", path.display()),
                Err(e) => println!("Failed to perform extration: {}", e),
            }
        }
    }
}
