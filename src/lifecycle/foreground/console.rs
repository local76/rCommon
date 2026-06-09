//! Advanced console / foreground lifecycle helpers.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).
//! 
//! Ported from trance (saver_win32.rs) and common patterns across apps.
//! Includes high contrast, thread execution state (prevent sleep), console titles,
//! screensaver control (for apps that manage them), and rect queries.
//!
//! Many delegate to library sys_info where possible (e.g. accent, dark mode, power).
//!
//! For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/ARCHITECTURE.md).
//! Cross-platform with native features and platform-specific stubs.

#[cfg(windows)]
use windows_sys::Win32::System::Console::{GetConsoleTitleW, SetConsoleTitleW};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
    SPI_SETSCREENSAVEACTIVE, SPI_SETSCREENSAVETIMEOUT,
};

/// High contrast detection (SPI).
/// Re-exported from Platform Native sys_info module to maintain clean taxonomy layers.
pub use crate::platform::native::sys_info::query_high_contrast;

/// Get console window rect (for positioning).
#[cfg(windows)]
pub fn console_window_rect() -> Option<crate::window::RECT> {
    unsafe {
        let hwnd = super::window::types::get_console_hwnd()?;
        super::window::types::get_console_window_rect(hwnd)
    }
}

#[cfg(not(windows))]
pub fn console_window_rect() -> Option<crate::window::RECT> {
    None
}

/// Update screensaver active state (for apps that manage screensaver behavior).
#[cfg(windows)]
pub fn update_screensaver_active(active: bool) {
    unsafe {
        SystemParametersInfoW(
            SPI_SETSCREENSAVEACTIVE,
            if active { 1 } else { 0 },
            std::ptr::null_mut(),
            SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
        );
    }
}

#[cfg(not(windows))]
pub fn update_screensaver_active(_active: bool) {}

/// Set screensaver timeout.
#[cfg(windows)]
pub fn update_screensaver_timeout(timeout_secs: u32) {
    unsafe {
        SystemParametersInfoW(
            SPI_SETSCREENSAVETIMEOUT,
            timeout_secs,
            std::ptr::null_mut(),
            SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
        );
    }
}

#[cfg(not(windows))]
pub fn update_screensaver_timeout(_timeout_secs: u32) {}

/// Get/set console title (cross platform where possible).
#[cfg(windows)]
pub fn get_console_title() -> std::io::Result<String> {
    let mut buf = [0u16; 512];
    let len = unsafe { GetConsoleTitleW(buf.as_mut_ptr(), buf.len() as u32) };
    if len == 0 {
        return Err(std::io::Error::last_os_error());
    }
    let safe_len = (len as usize).min(buf.len());
    Ok(String::from_utf16_lossy(&buf[..safe_len]))
}

#[cfg(windows)]
pub fn set_console_title(title: &str) -> std::io::Result<()> {
    let title_w: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let ok = unsafe { SetConsoleTitleW(title_w.as_ptr()) };
    if ok == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn get_console_title() -> std::io::Result<String> {
    Ok(std::env::var("TERM").unwrap_or_default())
}

#[cfg(not(windows))]
pub fn set_console_title(_title: &str) -> std::io::Result<()> {
    Ok(())
}

/// Attempts to hide the console's vertical scrollbar by sizing the screen buffer
/// exactly to the current visible window rect. This removes the native terminal
/// scrollbar UI on the right for TUI apps (the app manages its own scrolling
/// for lists, help, markdown viewers, etc. via keys/mouse).
///
/// Best called once after SetSize + entering alternate/raw mode.
pub fn hide_console_scrollbar() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::System::Console::{
            GetConsoleScreenBufferInfo, GetStdHandle, SetConsoleScreenBufferSize,
            CONSOLE_SCREEN_BUFFER_INFO, COORD, STD_OUTPUT_HANDLE,
        };

        let out_handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if out_handle.is_null() {
            return;
        }

        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
        if GetConsoleScreenBufferInfo(out_handle, &mut csbi) != 0 {
            let width = csbi.srWindow.Right - csbi.srWindow.Left + 1;
            let height = csbi.srWindow.Bottom - csbi.srWindow.Top + 1;
            let size = COORD { X: width, Y: height };
            // Buffer sized to viewport == no extra scrollback lines visible => scrollbar disappears.
            let _ = SetConsoleScreenBufferSize(out_handle, size);
        }
    }
}