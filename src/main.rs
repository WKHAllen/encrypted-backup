use clap::{Parser, Subcommand};
use glob::Pattern;
use std::path::{Path, PathBuf};

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
        /// Path to save the backup to
        #[clap(short, long, required = true, value_parser = validate_output_path)]
        output_path: PathBuf,
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

fn validate_output_path(output_path_str: &str) -> Result<PathBuf, String> {
    let output_path = PathBuf::from(output_path_str);

    if output_path.exists() {
        Err(format!("Path already exists: {}", output_path_str))
    } else if !output_path.parent().unwrap_or(Path::new("")).exists() {
        Err(format!("Parent path does not exist: {}", output_path_str))
    } else {
        Ok(output_path)
    }
}

fn main() {
    let cli = Cli::parse();
    dbg!(cli);
}
