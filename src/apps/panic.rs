//! Graceful panic handler hook for TUI applications.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground).

use std::panic::PanicHookInfo;
#[cfg(feature = "event-log")]
use std::backtrace::Backtrace;

/// Registers a custom panic hook that cleans up the TUI terminal state before printing the error.
///
/// If a TUI application panics while raw mode is active or inside an alternate screen,
/// this hook will restore the standard terminal mode, output a formatted panic report to stderr,
/// and log the full backtrace to the diagnostics log file.
pub fn set_tui_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        handle_tui_panic(panic_info);
    }));
}

fn handle_tui_panic(panic_info: &PanicHookInfo) {
    // 1. Clean up the terminal raw mode and alternate screen
    #[cfg(feature = "widgets")]
    {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
    }

    // 2. Extract panic message
    let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        *s
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        s.as_str()
    } else {
        "Box<dyn Any>"
    };

    // 3. Extract source location
    let location = panic_info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "unknown location".to_string());

    // 4. Capture backtrace and log to diagnostic log file
    #[cfg(feature = "event-log")]
    let backtrace = Backtrace::capture();
    #[cfg(feature = "event-log")]
    let crash_report = format!(
        "Application panicked at {}: {}\n\nBacktrace:\n{}",
        location, msg, backtrace
    );
    #[cfg(feature = "event-log")]
    crate::apps::file_log::log_message("PANIC", &crash_report);

    // 5. Print a clean, user-friendly report to stderr
    eprintln!("\n══════════════════════════════════════════════════════════════");
    eprintln!(" ⚠️  FATAL ERROR: Application Panicked");
    eprintln!("══════════════════════════════════════════════════════════════");
    eprintln!("Location : {}", location);
    eprintln!("Error    : {}", msg);
    eprintln!("══════════════════════════════════════════════════════════════");
    eprintln!("A detailed crash report has been saved to the diagnostics log.");
    eprintln!("Restored terminal to normal mode. Exiting.\n");

    // Force terminate the process immediately to avoid deadlock or double panics
    std::process::exit(1);
}
