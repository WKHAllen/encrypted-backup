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
pub fn init(debug: bool) -> Result<(), SetLoggerError> {
    let max_level = match debug {
        true => LevelFilter::Debug,
        false => LevelFilter::Warn,
    };

    log::set_logger(&LOGGER).map(|()| log::set_max_level(max_level))
}
