// =====================================================
// rCommon - Shared utility library for the local76 rApps ecosystem
// Organized according to the 4-layer taxonomy:
//
// 1. Interface (Presentation Layer)
//    - CLI, TUI, GUI-Native, GUI-Custom/Game-Engine, Headless/API
//
// 2. Execution State (Lifecycle)
//    - Foreground Applications, Background Processes
//
// 3. Platform & Architecture (Deployment)
//    - Native (Windows/Linux), Web, Mobile, Embedded
//
// 4. System Role (Purpose)
//    - System Software (infrastructure), Application Software (task-oriented)
//
// core/ is the only layer that must remain neutral and usable by any combination.
//
// This structure prevents accidental coupling between concerns
// (e.g., a TUI effect type being changed in a way that breaks a background service).
// =====================================================
//
// MIGRATION GUIDE FOR CONSUMERS (Moving off deprecated rcommon::win32):
// - rcommon::win32::SingleInstanceGuard -> rcommon::lifecycle::foreground::guard::SingleInstanceGuard
// - rcommon::win32::hide_console_at_startup -> rcommon::lifecycle::foreground::window::hide_console_at_startup
// - rcommon::win32::query_dark_mode -> rcommon::platform::native::sys_info::query_dark_mode
// - rcommon::win32::TerminalCell -> rcommon::core::TerminalCell
// - rcommon::win32::read_string -> rcommon::platform::native::reg::read_string
// - rcommon::win32::get_packages_breakdown -> rcommon::role::application::packages::get_packages_breakdown
//
// =====================================================


/// Core neutral primitives (TerminalCell, LcgRng, DashboardInfo, etc.).
/// Safe to use from CLI, TUI, background services, or future targets.
pub mod core;
pub mod error;

pub use error::{RcommonError, Result as RcommonResult};
#[cfg(feature = "effects")]
pub use interface::tui::screensaver::{Screensaver, ScreensaverRenderer};

// =====================================================
// 1. Interface (Presentation Layer)
// =====================================================
pub mod interface;

// Backward compatibility re-exports (so existing code like `rcommon::widgets` still works)
#[cfg(feature = "widgets")]
pub use interface::tui::widgets;
#[cfg(feature = "effects")]
pub use interface::tui::effects;
pub use interface::tui::text;
#[cfg(feature = "effects")]
pub use interface::tui::screensaver;
#[cfg(feature = "gui")]
pub use interface::gui::gui as gui;

// =====================================================
// 2. Execution State (Lifecycle)
// =====================================================
pub mod lifecycle;

// Backward compat re-exports
#[cfg(feature = "window")]
pub use lifecycle::foreground::window;
#[cfg(feature = "window")]
pub use lifecycle::foreground::guard;
#[cfg(feature = "service")]
pub use lifecycle::background::service;
#[cfg(feature = "event-log")]
pub use lifecycle::background::event_log;
#[cfg(feature = "notification")]
pub use lifecycle::background::notification;
#[cfg(feature = "clipboard")]
pub use lifecycle::background::clipboard;
pub use lifecycle::background::daemon;

// =====================================================
// 3. Platform & Architecture (Deployment)
// =====================================================
pub mod platform;

// Backward compat
pub use platform::native::sys_info;
pub use platform::native::reg;

// =====================================================
// 4. System Role (Purpose)
// =====================================================
pub mod role;

// Backward compat for application role
pub use role::application::rgb;
pub use role::application::game;
pub use role::application::packages::{
    count_scoop, count_choco, count_npm, count_steam, count_ms_store, count_native, count_winget, count_dpkg, count_pacman,
    count_flatpak, count_snap, PackageManager, PACKAGE_MANAGERS, get_packages_breakdown
};

// Platform native additions (monitors)
pub use platform::native::monitors::{get_monitors_summary, get_all_monitors};

// Lifecycle foreground additions (advanced console helpers and window)
pub use lifecycle::foreground::set_tui_panic_hook;

#[cfg(feature = "window")]
pub use lifecycle::foreground::window::hide_console_at_startup;
#[cfg(feature = "window")]
#[allow(deprecated)] // Intentional: Re-exporting legacy conhost relaunch helpers for backward compatibility with older rApps
pub use lifecycle::foreground::window::{
    RECT, MONITORINFO, COORD, SMALL_RECT, CONSOLE_SELECTION_INFO, POINT,
    get_console_rect, get_window_rect, set_window_pos, center_console_window, query_cursor_pos,
    relaunch_in_conhost_if_needed, should_relaunch_in_conhost, relaunch_in_conhost,
    is_console_focused,
    BorderlessConsole, ConsoleTitleGuard,
    SingleInstanceGuard
};
#[cfg(feature = "window")]
pub use lifecycle::foreground::console::{
    query_high_contrast, console_window_rect, update_screensaver_active,
    update_screensaver_timeout, get_console_title, set_console_title,
    hide_console_scrollbar
};

// Core enhancements
pub use core::SystemInfo;
pub use platform::native::sys_info::get_system_info;

// GPU compute enhancements
#[cfg(feature = "gpu")]
pub use platform::native::gpu::{init_headless_gpu, run_compute_shader};

// Theme enhancements
pub use platform::native::sys_info::{SystemTheme, query_accent_color, query_system_theme};



// =====================================================
// Prep for multi-crate future (per taxonomy sections)
// =====================================================
// When ready, this can become a Cargo workspace:
// [workspace]
// members = ["core", "interface/tui", "lifecycle", "platform/native", "role/*"]
// Each section crate would re-export from its modules.
// For now, single crate keeps git-dep + [patch] simple for r* apps.
// Update consumers gradually to use taxonomy paths (e.g. rcommon::interface::tui).

/// Extension trait to expose background daemon services over IPC.
#[cfg(feature = "interface-api")]
pub trait DaemonIpcExt {
    /// Exposes the daemon service via IPC using the Headless/API layer.
    /// Binds to a local named pipe (Windows) or domain socket (Unix) named after the daemon,
    /// and listens for requests.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rcommon::lifecycle::background::daemon::{DaemonService, DaemonConfig};
    /// use rcommon::interface::api::{IpcResponse, IpcRequest};
    /// use rcommon::DaemonIpcExt;
    ///
    /// let config = DaemonConfig::new("my_daemon");
    /// let daemon = DaemonService::bootstrap(config).unwrap();
    ///
    /// daemon.run_ipc_server(|req| {
    ///     match req.command.as_str() {
    ///         "status" => IpcResponse::ok("running", ""),
    ///         _ => IpcResponse::err("unknown command"),
    ///     }
    /// }).unwrap();
    /// ```
    fn run_ipc_server<F>(&self, handler: F) -> Result<(), crate::error::RcommonError>
    where
        F: Fn(interface::api::IpcRequest) -> interface::api::IpcResponse;
}

#[cfg(feature = "interface-api")]
impl DaemonIpcExt for lifecycle::background::daemon::DaemonService {
    fn run_ipc_server<F>(&self, handler: F) -> Result<(), crate::error::RcommonError>
    where
        F: Fn(interface::api::IpcRequest) -> interface::api::IpcResponse
    {
        let host = interface::api::IpcServiceHost::new(self.name())?;
        host.run(handler);
        Ok(())
    }
}