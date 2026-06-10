//! Console visibility and activation state helper functions.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

#[cfg(target_os = "windows")]
use super::types::{get_console_hwnd, SW_HIDE};

/// Hides the console window early at startup (common pattern used by template,
/// scout, pulse, ignite, etc.). Returns the raw hwnd on Windows if successful.
/// Centralized here so apps can call `library::window::hide_console_at_startup();`
pub fn hide_console_at_startup() -> Option<*mut std::ffi::c_void> {
    #[cfg(target_os = "windows")]
    {
        #[cfg(feature = "sys-info")]
        {
        let (_, terminal) = crate::sys_info::query_shell_and_terminal();
        if terminal != "Windows Console Host" {
            return None;
        }
        }

        let hwnd = unsafe { get_console_hwnd() }?;
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                hwnd,
                SW_HIDE,
            );
        }
        Some(hwnd)
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// Re-shows the console window and brings it to the foreground on Windows.
pub fn show_console_window() {
    #[cfg(target_os = "windows")]
    {
        if let Some(hwnd) = unsafe { get_console_hwnd() } {
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                    hwnd,
                    windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW,
                );
                windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
            }
        }
    }
}

/// Query if the console window is currently the active foreground window.
/// Classification: Lifecycle (Foreground) + Platform (Native).
pub fn is_console_focused() -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = match get_console_hwnd() {
            Some(h) => h,
            None => return false,
        };
        let fg = windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
        hwnd == fg
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_focused() {
        let _ = is_console_focused();
    }
}
