//! Generic file-based and system event logging.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Background) + Platform (Native).

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

static EVENT_LOG_ENABLED: AtomicBool = AtomicBool::new(false);
static EVENT_SOURCE: OnceLock<String> = OnceLock::new();
static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();
static LOG_APP_NAME: OnceLock<String> = OnceLock::new();

/// Set the event source name for Windows Event Log / Syslog.
pub fn set_event_source(name: &str) {
    // If it's already set, we ignore or replace. Since OnceLock can only be set once, we just allow setting it once.
    let _ = EVENT_SOURCE.set(name.to_string());
}

fn get_event_source() -> &'static str {
    EVENT_SOURCE.get_or_init(|| "rApp".to_string())
}

/// Enable or disable Windows Event Log syncing globally.
pub fn set_event_log_enabled(enabled: bool) {
    EVENT_LOG_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if Windows Event Log syncing is globally enabled.
pub fn is_event_log_enabled() -> bool {
    EVENT_LOG_ENABLED.load(Ordering::Relaxed)
}

/// Set the per-app log file folder name (e.g. `"rfetch"`, `"rmonitor"`, `"rwifi"`, `"rstart"`).
/// Subsequent calls to `log_message` will write to `%APPDATA%\<app_name>\log.txt`.
/// Subsequent calls to `get_appdata_log_path` will return the same path.
///
/// Once set, it cannot be changed (uses an internal `OnceLock`). Calling this more
/// than once silently no-ops.
pub fn set_log_app_name(name: &str) {
    let _ = LOG_APP_NAME.set(name.to_string());
}

fn get_log_app_name() -> &'static str {
    LOG_APP_NAME.get_or_init(|| "rcommon".to_string())
}

/// Helper to resolve the standard AppData folder for diagnostics logging.
/// Path: `%APPDATA%\<app_name>\log.txt` where `<app_name>` is set via [`set_log_app_name`].
/// Default folder name (before `set_log_app_name` is called) is `"rcommon"`.
/// (Pre-4.2.0 default was the rTemplate-specific string `"rTmp"`, a leftover
/// from before the 2026-06-08 install-path-alignment sprint. Every consumer
/// app calls `set_log_app_name` at startup, so the default is dead code
/// in practice — the rename to `"rcommon"` is purely a cleanliness fix.)
pub fn get_appdata_log_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var("APPDATA").ok().map(|appdata| {
            std::path::PathBuf::from(appdata)
                .join(get_log_app_name())
                .join("log.txt")
        })
    }
    #[cfg(not(windows))]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME").ok().map(|home| {
                    PathBuf::from(home).join(".local").join("share")
                })
            });
        base.map(|b| b.join(get_log_app_name()).join("log.txt"))
    }
}

fn get_log_file() -> &'static Mutex<Option<File>> {
    LOG_FILE.get_or_init(|| {
        let file_opt = get_appdata_log_path().and_then(|path| {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .ok()
        });
        Mutex::new(file_opt)
    })
}

/// Thread-safe silent logger helper that appends diagnostic logs to a local file.
pub fn log_message(level: &str, msg: &str) {
    let mutex = get_log_file();
    if let Ok(mut guard) = mutex.lock() {
        if let Some(ref mut file) = *guard {
            let epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let _ = writeln!(file, "[{}] [{}] {}", epoch, level, msg);
        }
    }

    if is_event_log_enabled() {
        let event_type = match level {
            "ERROR" | "PANIC" => super::event_log::EVENTLOG_ERROR_TYPE,
            "WARNING" => super::event_log::EVENTLOG_WARNING_TYPE,
            _ => super::event_log::EVENTLOG_INFORMATION_TYPE,
        };
        // Write the event using the native event_log module function
        super::event_log::log_system_event(
            get_event_source(),
            event_type,
            super::event_log::EVENT_ID_USER_ACTION,
            msg,
        );
    }
}
