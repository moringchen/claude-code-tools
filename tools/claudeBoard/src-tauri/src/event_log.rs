use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;

fn log_write_lock() -> &'static Mutex<()> {
    static LOG_WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOG_WRITE_LOCK.get_or_init(|| Mutex::new(()))
}

fn log_path() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".claude-board")
        .join("events.log")
}

fn rotate_if_needed(path: &Path) -> io::Result<()> {
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.len() > MAX_LOG_BYTES {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn local_timestamp_string(explicit_timestamp: Option<&str>) -> String {
    if let Some(timestamp) = explicit_timestamp {
        return timestamp.to_string();
    }

    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    now.format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

fn append_event_log_inner(path: &Path, line: &str, explicit_timestamp: Option<&str>) -> io::Result<()> {
    let _guard = log_write_lock().lock().unwrap();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    rotate_if_needed(path)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{} {}", local_timestamp_string(explicit_timestamp), line)?;
    file.flush()?;
    Ok(())
}

pub fn append_event_log(line: &str) {
    let path = log_path();
    if let Err(error) = append_event_log_inner(&path, line, None) {
        eprintln!("[claudeBoard] event log write failed: {error}");
    }
}

pub fn reset_event_log() -> io::Result<()> {
    let path = log_path();
    let _guard = log_write_lock().lock().unwrap();
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn append_event_log_to_path(path: &Path, line: &str, explicit_timestamp: Option<&str>) -> io::Result<()> {
    append_event_log_inner(path, line, explicit_timestamp)
}
