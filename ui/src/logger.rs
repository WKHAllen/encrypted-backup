//! Application-level logging configuration.

use log::{LevelFilter, SetLoggerError};
use tokio::sync::mpsc::UnboundedSender;

/// The application-level logger.
struct BackupLogger(UnboundedSender<String>);

impl log::Log for BackupLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.target().starts_with("backup::")
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let _ = self.0.send(format!(
                "[{}] {}",
                chrono::Local::now().format("%a %Y-%m-%d %H:%M:%S%.3f"),
                record.args()
            ));
        }
    }

    fn flush(&self) {}
}

/// Initializes logging.
///
/// # Errors
///
/// This will return an error if the logger has already been initialized.
pub fn init_logger(debug: bool, sender: UnboundedSender<String>) -> Result<(), SetLoggerError> {
    let max_level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    };

    log::set_boxed_logger(Box::new(BackupLogger(sender))).map(|()| log::set_max_level(max_level))
}
