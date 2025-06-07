use chrono::Local;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::{LazyLock, Mutex};

const DATE_FORMAT: &str = "%Y/%m/%d %H:%M:%S";

struct Logger {
    writer: Mutex<BufWriter<std::fs::File>>,
    log_debug_enabled: bool,
}

impl Logger {
    fn new(log_file_name: &str, log_debug_enabled: bool) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_name)?;

        Ok(Logger {
            writer: Mutex::new(BufWriter::new(file)),
            log_debug_enabled,
        })
    }

    fn log(&self, level: &str, message: &str) {
        let timestamp = Local::now().format(DATE_FORMAT).to_string();
        let log_line = format!("[{}] [{}] [RUST] {}\n", level, timestamp, message);

        if let Ok(mut writer) = self.writer.lock() {
            let _ = writer.write_all(log_line.as_bytes());
            let _ = writer.flush();
        }
    }
}

static LOGGER: LazyLock<Mutex<Option<Logger>>> = LazyLock::new(|| Mutex::new(None));

pub fn logger_init<P: AsRef<Path>>(log_file_name: P, log_debug_enabled: bool) -> std::io::Result<()> {
    let logger = Logger::new(log_file_name.as_ref().to_str().unwrap(), log_debug_enabled)?;
    let mut global_logger = LOGGER.lock().unwrap();
    *global_logger = Some(logger);
    Ok(())
}

pub fn debug(args: std::fmt::Arguments) {
    if let Some(logger) = LOGGER.lock().unwrap().as_ref() {
        if logger.log_debug_enabled {
            logger.log("DEBUG", &format!("{}", args));
        }
    }
}

pub fn error(args: std::fmt::Arguments) {
    if let Some(logger) = LOGGER.lock().unwrap().as_ref() {
        logger.log("ERROR", &format!("{}", args));
    }
}

#[macro_export]
macro_rules! dbeer_debug {
    ($($arg:tt)*) => {
        $crate::debug(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! dbeer_error {
    ($($arg:tt)*) => {
        $crate::error(format_args!($($arg)*))
    };
}
