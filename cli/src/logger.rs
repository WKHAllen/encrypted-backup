//! Application-level logging configuration.

use log::{LevelFilter, SetLoggerError};

/// The application-level logger.
struct BackupLogger;

impl log::Log for BackupLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!(
                "[{}] {}",
                chrono::Local::now().format("%a %Y-%m-%d %H:%M:%S%.3f"),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

/// The global logging instance.
static LOGGER: BackupLogger = BackupLogger;

/// Initializes logging.
///
/// # Errors
///
/// This will return an error if the logger has already been initialized.
pub fn init_logger(debug: bool) -> Result<(), SetLoggerError> {
    let max_level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    };

    log::set_logger(&LOGGER).map(|()| log::set_max_level(max_level))
}
