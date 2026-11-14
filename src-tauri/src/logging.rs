use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::Manager;

const LOG_FILE: &str = "cliphistory.log";
const MAX_LOG_BYTES: u64 = 1_000_000;

static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();
static WRITE_LOCK: Mutex<()> = Mutex::new(());

pub fn init(app: &tauri::AppHandle) {
    let Ok(data_dir) = app.path().app_data_dir() else {
        return;
    };
    let dir = data_dir.join("logs");
    if fs::create_dir_all(&dir).is_err() {
        return;
    }

    rotate_if_needed(&dir.join(LOG_FILE));
    let _ = LOG_DIR.set(dir);
    info("application started");
}

pub fn dir() -> Option<PathBuf> {
    LOG_DIR.get().cloned()
}

fn path() -> Option<PathBuf> {
    LOG_DIR.get().map(|dir| dir.join(LOG_FILE))
}

pub fn info(message: &str) {
    write("INFO", message);
}

pub fn warn(message: &str) {
    write("WARN", message);
}

pub fn error(message: &str) {
    write("ERROR", message);
}

fn write(level: &str, message: &str) {
    // Keep the existing terminal diagnostics, but also persist a redacted log for users.
    eprintln!("cliphistory: {message}");

    let Some(path) = path() else {
        return;
    };
    let Ok(_guard) = WRITE_LOCK.lock() else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let _ = writeln!(file, "[{timestamp}] {level} {message}");
}

pub fn tail(limit: usize) -> String {
    let Some(path) = path() else {
        return String::new();
    };
    let Ok(contents) = fs::read_to_string(path) else {
        return String::new();
    };
    tail_contents(&contents, limit)
}

fn tail_contents(contents: &str, limit: usize) -> String {
    let lines: Vec<&str> = contents.lines().collect();
    lines[lines.len().saturating_sub(limit)..].join("\n")
}

fn rotate_if_needed(path: &Path) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    if metadata.len() < MAX_LOG_BYTES {
        return;
    }

    let rotated = path.with_extension("log.1");
    let _ = fs::remove_file(&rotated);
    let _ = fs::rename(path, rotated);
}

#[cfg(test)]
mod tests {
    use super::tail_contents;

    #[test]
    fn returns_only_the_last_lines() {
        let contents = "one\ntwo\nthree\nfour\n";
        assert_eq!(tail_contents(contents, 2), "three\nfour");
    }

    #[test]
    fn handles_a_small_log() {
        assert_eq!(tail_contents("only line", 20), "only line");
        assert_eq!(tail_contents("", 20), "");
    }
}
