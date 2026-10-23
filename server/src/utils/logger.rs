use crate::utils::time::now_iso;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, Once, OnceLock};

static FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
static INIT: Once = Once::new();

struct FileLogger;

static LOGGER: FileLogger = FileLogger;

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format_line(record.level().as_str(), &record.args().to_string());
        eprint!("{line}");
        if let Some(file) = FILE.get() {
            if let Ok(mut handle) = file.lock() {
                let _ = handle.write_all(line.as_bytes());
                let _ = handle.flush();
            }
        }
    }

    fn flush(&self) {
        if let Some(file) = FILE.get() {
            if let Ok(mut handle) = file.lock() {
                let _ = handle.flush();
            }
        }
    }
}

pub fn format_line(level: &str, message: &str) -> String {
    format!("{} {level:<5} {message}\n", now_iso())
}

/// Opens `server.log` (or `AXON_LOG_FILE`) and sends every log line there and to stderr.
pub fn init() {
    INIT.call_once(|| {
        let path = std::env::var("AXON_LOG_FILE").unwrap_or_else(|_| "server.log".to_string());
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => {
                let _ = FILE.set(Mutex::new(file));
            }
            Err(error) => {
                eprintln!("Could not open log file {path}: {error}");
            }
        }
        let _ = log::set_logger(&LOGGER);
        log::set_max_level(log::LevelFilter::Debug);
    });
}

pub fn info(message: impl std::fmt::Display) {
    log::info!("{message}");
}

pub fn warn(message: impl std::fmt::Display) {
    log::warn!("{message}");
}

pub fn error(message: impl std::fmt::Display) {
    log::error!("{message}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_contains_the_level_and_message() {
        let line = format_line("INFO", "node joined");
        assert!(line.contains("INFO"));
        assert!(line.contains("node joined"));
        assert!(line.ends_with('\n'));
    }
}
