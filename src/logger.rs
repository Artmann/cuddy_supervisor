use colored::*;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct ColoredLogger;

impl log::Log for ColoredLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level = match record.level() {
                Level::Error => record.level().to_string().red(),
                Level::Warn => record.level().to_string().yellow(),
                Level::Info => record.level().to_string().green(),
                Level::Debug => record.level().to_string().blue(),
                Level::Trace => record.level().to_string().purple(),
            };
            println!("{} - {}", level, record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: ColoredLogger = ColoredLogger;

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))
}
