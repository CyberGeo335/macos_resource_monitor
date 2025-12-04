use crate::metrics::ResourceSnapshot;
use chrono::Utc;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Logger {
    log_file_path: PathBuf,
}

#[derive(Serialize)]
struct LogEvent<'a> {
    timestamp: String,
    level: &'a str,
    message: &'a str,
}

impl Logger {
    pub fn new<P: AsRef<Path>>(log_file_path: P) -> std::io::Result<Self> {
        let path = log_file_path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Logger { log_file_path: path })
    }

    fn open_append(&self) -> std::io::Result<std::fs::File> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
    }

    pub fn log_snapshot(&self, snapshot: &ResourceSnapshot) -> std::io::Result<()> {
        let mut file = self.open_append()?;
        let line = serde_json::to_string(snapshot)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    pub fn log_event(&self, level: &str, message: &str) -> std::io::Result<()> {
        let mut file = self.open_append()?;
        let event = LogEvent {
            timestamp: Utc::now().to_rfc3339(),
            level,
            message,
        };
        let line = serde_json::to_string(&event)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }
}
