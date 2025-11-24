#![allow(unused)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;

static LOG_FILE: OnceLock<Mutex<Option<std::fs::File>>> = OnceLock::new();

#[cfg(debug_assertions)]
fn log_file() -> &'static Mutex<Option<std::fs::File>> {
    LOG_FILE.get_or_init(|| {
        let path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("heatseeker-debug.log");
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .ok();
        Mutex::new(file)
    })
}

#[cfg(not(debug_assertions))]
fn log_file() -> &'static Mutex<Option<std::fs::File>> {
    static NO_LOG: OnceLock<Mutex<Option<std::fs::File>>> = OnceLock::new();
    NO_LOG.get_or_init(|| Mutex::new(None))
}

pub fn init_logging() {
    #[cfg(debug_assertions)]
    #[cfg(not(test))]
    {
        let mut guard = log_file().lock().unwrap();
        if let Some(ref mut f) = *guard {
            let _ = writeln!(
                f,
                "--- heatseeker debug start pid={} ---",
                std::process::id()
            );
            let _ = f.flush();
        }
    }
}

pub fn log_bytes(tag: &str, bytes: &[u8]) {
    #[cfg(debug_assertions)]
    #[cfg(not(test))]
    {
        let mut guard = log_file().lock().unwrap();
        if let Some(ref mut f) = *guard {
            let hex: Vec<String> = bytes.iter().map(|b| format!("{b:02X}")).collect();
            let printable: String = bytes
                .iter()
                .map(|b| {
                    let c = *b as char;
                    if c.is_control() { '.' } else { c }
                })
                .collect();
            let _ = writeln!(f, "[{}] hex={} text={}", tag, hex.join(" "), printable);
            let _ = f.flush();
        }
    }
}

pub fn log_line(msg: &str) {
    #[cfg(debug_assertions)]
    #[cfg(not(test))]
    {
        let mut guard = log_file().lock().unwrap();
        if let Some(ref mut f) = *guard {
            let _ = writeln!(f, "{msg}");
            let _ = f.flush();
        }
    }
}
