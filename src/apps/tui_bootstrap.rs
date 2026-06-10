//! One-shot TUI bootstrap: enable raw mode, alt screen, mouse capture, console sizing.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground).
//!
//! Encapsulates the ~25-line raw-mode + alt-screen + size + borderless dance that
//! starts every r* TUI's `main()`. Returns a `Terminal` ready for rendering.

use std::sync::atomic::{AtomicBool, Ordering};

static APP_SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

/// Signal that the TUI application is shutting down.
pub fn set_app_shutting_down(val: bool) {
    APP_SHUTTING_DOWN.store(val, Ordering::Relaxed);
}

/// Check if the application is shutting down (for background threads).
pub fn is_app_shutting_down() -> bool {
    APP_SHUTTING_DOWN.load(Ordering::Relaxed)
}

#[cfg(all(target_os = "windows", feature = "widgets"))]
unsafe extern "system" fn tui_ctrl_handler(ctrl_type: u32) -> windows_sys::Win32::Foundation::BOOL {
    use windows_sys::Win32::Foundation::FALSE;
    use windows_sys::Win32::System::Console::{
        CTRL_C_EVENT, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT
    };

    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT => {
            // Signal background threads to stop
            set_app_shutting_down(true);

            // Clear notifications
            #[cfg(feature = "notification")]
            crate::lifecycle::background::notification::clear_my_toast_notifications();

            // Restore terminal state
            let _ = crossterm::terminal::disable_raw_mode();
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::event::DisableMouseCapture
            );

            // Return FALSE so the default handler still terminates the process.
            FALSE
        }
        _ => FALSE,
    }
}

#[cfg(feature = "widgets")]
mod imp {
    use std::io;

    use ratatui::{Terminal, backend::TermwizBackend};
    use crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetSize, disable_raw_mode, enable_raw_mode},
        event::{EnableMouseCapture, DisableMouseCapture},
    };

    use crate::lifecycle::foreground::panic::set_tui_panic_hook;
    use crate::lifecycle::foreground::window::{BorderlessConsole, ConsoleTitleGuard, SingleInstanceGuard, center_console_window};

    /// Configuration for `bootstrap_tui`.
    #[derive(Debug, Clone)]
    pub struct TuiBootstrapConfig {
        /// Window title (used for the console tab/title and the `ConsoleTitleGuard`).
        pub title: &'static str,
        /// Whether to enforce a 100x35 minimum via `SetSize`.
        pub size: (u16, u16),
        /// If true, install a `SingleInstanceGuard` and exit on conflict.
        pub enforce_single_instance: bool,
        /// If true, enable the borderless console window and skip centering.
        pub borderless: bool,
        /// Whether to install the `set_tui_panic_hook` (recommended).
        pub install_panic_hook: bool,
    }

    impl TuiBootstrapConfig {
        pub fn new(title: &'static str) -> Self {
            Self {
                title,
                size: (100, 35),
                enforce_single_instance: true,
                borderless: false,
                install_panic_hook: true,
            }
        }
    }

    /// Automatically disables raw mode and restores screen settings on drop.
    pub struct TerminalGuard {
        active: bool,
    }

    impl TerminalGuard {
        pub fn new() -> Self {
            Self { active: true }
        }
        pub fn deactivate(&mut self) {
            self.active = false;
        }
    }

    impl Default for TerminalGuard {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Drop for TerminalGuard {
        fn drop(&mut self) {
            if self.active {
                let _ = disable_raw_mode();
                let _ = execute!(
                    io::stdout(),
                    LeaveAlternateScreen,
                    DisableMouseCapture
                );
            }
        }
    }

    /// All Drop guards returned by `bootstrap_tui` so the caller can keep them alive.
    pub struct TuiGuards {
        /// Set if `enforce_single_instance` is true.
        pub _instance_guard: Option<SingleInstanceGuard>,
        /// Always set while the TUI is running.
        pub _title_guard: ConsoleTitleGuard,
        /// Set if `borderless` is true.
        pub _borderless: Option<BorderlessConsole>,
        /// Restores terminal configuration automatically on drop.
        pub _terminal_guard: TerminalGuard,
    }

    /// Enable raw mode, alt screen, mouse capture, sizing, optional single-instance & borderless.
    /// Returns the Terminal + Drop guards. The caller should hold onto `guards` until shutdown.
    pub fn bootstrap_tui(
        config: TuiBootstrapConfig,
    ) -> io::Result<(Terminal<TermwizBackend>, TuiGuards)> {
        super::set_app_shutting_down(false);

        #[cfg(target_os = "windows")]
        unsafe {
            let _ = windows_sys::Win32::System::Console::SetConsoleCtrlHandler(
                Some(super::tui_ctrl_handler),
                windows_sys::Win32::Foundation::TRUE,
            );
        }

        if config.install_panic_hook {
            set_tui_panic_hook();
        }

        let _instance_guard = if config.enforce_single_instance {
            Some(SingleInstanceGuard::try_new_or_exit(config.title))
        } else {
            None
        };

        let _title_guard = ConsoleTitleGuard::new(config.title);

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetSize(config.size.0, config.size.1));
        if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            let _ = disable_raw_mode();
            return Err(e);
        }

        let _borderless = if config.borderless {
            Some(BorderlessConsole::enable())
        } else {
            None
        };
        // Allow console size/style changes to propagate to the buffer
        std::thread::sleep(std::time::Duration::from_millis(50));

        if _borderless.is_none() {
            center_console_window();
        }

        let backend = TermwizBackend::new().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let _terminal_guard = TerminalGuard::new();

        Ok((
            terminal,
            TuiGuards {
                _instance_guard,
                _title_guard,
                _borderless,
                _terminal_guard,
            },
        ))
    }

    /// Restore raw-mode terminal state. Call this at the end of `main` (or in a Drop).
    pub fn shutdown_tui(
        _terminal: &mut Terminal<TermwizBackend>,
    ) -> io::Result<()> {
        super::set_app_shutting_down(true);

        #[cfg(feature = "notification")]
        crate::apps::notification::clear_my_toast_notifications();

        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        Ok(())
    }
}

#[cfg(feature = "widgets")]
pub use imp::*;
