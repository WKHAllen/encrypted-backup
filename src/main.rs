mod backup;

use clap::{Parser, Subcommand};
use glob::Pattern;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Backs up and encrypts files and directories
    Backup {
        /// Comma-separated paths to include in the backup
        #[clap(short, long, required = true, multiple_values = true, value_delimiter = ',', value_parser = validate_path)]
        include_paths: Vec<PathBuf>,
        /// Comma-separated globs to exclude from the backup
        #[clap(short, long, multiple_values = true, value_delimiter = ',', value_parser = validate_glob)]
        exclude_globs: Vec<Pattern>,
        /// Directory to save the backup to
        #[clap(short, long, required = true, value_parser = validate_dir)]
        output_dir: PathBuf,
        /// Name of the backup file
        #[clap(short, long, value_parser)]
        name: String,
    },
    /// Decrypts and extracts an encrypted backup
    Extract {
        /// Path to the encrypted backup
        #[clap(value_parser = validate_file)]
        path: PathBuf,
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup {
            include_paths,
            exclude_globs,
            output_dir,
            name,
        } => match backup::backup(&include_paths, &exclude_globs, &output_dir, &name) {
            Ok(path) => println!("Successfully backed up to {}", path.display()),
            Err(e) => println!("Failed to perform backup: {}", e),
        },
        Commands::Extract { path } => match backup::extract(&path) {
            Ok(path) => println!("Successfully extracted to {}", path.display()),
            Err(e) => println!("Failed to perform extration: {}", e),
        },
    }
}
