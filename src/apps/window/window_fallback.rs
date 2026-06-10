//! Fallback console and window management for other platform targets.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

use std::io;
use crate::apps::window::types::RECT;

pub fn get_console_rect() -> Option<RECT> {
    None
}

pub fn get_window_rect() -> Option<RECT> {
    None
}

pub fn set_window_pos(_x: i32, _y: i32) {}

pub fn center_console_window() {}

pub fn query_cursor_pos() -> Option<(i32, i32)> {
    None
}

pub fn should_relaunch_in_conhost() -> bool {
    false
}

pub fn relaunch_in_conhost() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Conhost relaunch is only supported on Windows",
    ))
}

#[allow(deprecated)]
pub fn relaunch_in_conhost_if_needed() {}

pub fn hide_console_at_startup() -> Option<*mut std::ffi::c_void> {
    None
}

pub fn show_console_window() {}

pub fn is_console_focused() -> bool {
    false
}

pub struct BorderlessConsole {}

impl BorderlessConsole {
    pub fn enable() -> Self {
        BorderlessConsole {}
    }
    pub fn enable_preserving_size() -> Self {
        BorderlessConsole {}
    }
}

pub struct ConsoleTitleGuard {}

impl ConsoleTitleGuard {
    pub fn new(_title: &str) -> Self {
        ConsoleTitleGuard {}
    }
}

impl Drop for ConsoleTitleGuard {
    fn drop(&mut self) {}
}
