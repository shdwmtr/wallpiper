use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::sync::Mutex;

use crate::config::{LOG_PATH, LOG_SAMPLE_INTERVAL, LOG_SAMPLE_WARMUP};

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

pub(crate) fn emit(msg: &str) {
    let mut guard = LOG_FILE.lock().unwrap();
    if guard.is_none() {
        *guard = OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_PATH)
            .ok();
    }
    let Some(file) = guard.as_mut() else {
        return;
    };
    if writeln!(file, "[pid={}] {msg}", std::process::id()).is_err() {
        *guard = None;
    }
}

pub(crate) fn should_sample(count: u64) -> bool {
    count <= LOG_SAMPLE_WARMUP || count.is_multiple_of(LOG_SAMPLE_INTERVAL)
}

macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logging::emit(&format!($($arg)*))
    };
}
pub(crate) use log;
