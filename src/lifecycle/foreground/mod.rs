//! Foreground Application lifecycle.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground).
//!
//! Part of Execution State (Lifecycle).
//! Things that require active user attention, console focus, etc.

// Foreground code moved here (window management, guards, console helpers).

#[cfg(feature = "window")]
pub mod window;
#[cfg(feature = "window")]
pub mod guard;
#[cfg(feature = "window")]
pub mod console;
#[cfg(feature = "window")]
pub mod identity;
#[cfg(all(feature = "window", feature = "widgets"))]
pub mod tui_bootstrap;
pub mod power_sync;
pub mod panic;
pub use panic::set_tui_panic_hook;

#[cfg(feature = "window")]
#[allow(deprecated)] // Intentional: Re-exporting legacy relaunch helpers for backward compatibility with older apps
pub use window::{
    RECT, MONITORINFO, COORD, SMALL_RECT, CONSOLE_SELECTION_INFO, POINT,
    get_console_rect, get_window_rect, set_window_pos, center_console_window, query_cursor_pos,
    relaunch_in_conhost_if_needed, should_relaunch_in_conhost, relaunch_in_conhost,
    hide_console_at_startup, is_console_focused,
    BorderlessConsole, ConsoleTitleGuard,
    SingleInstanceGuard,
    drag_to_move::WindowDrag,
};
#[cfg(feature = "window")]
pub use console::{
    console_window_rect, update_screensaver_active, update_screensaver_timeout, get_console_title, set_console_title, hide_console_scrollbar,
};
#[cfg(feature = "window")]
pub use identity::{hostname, os_str, user_host, username};
pub use power_sync::PowerThrottle;