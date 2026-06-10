//! Window positioning and geometry helper functions.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

use super::types::RECT;

#[cfg(target_os = "windows")]
use super::types::{get_console_hwnd, get_console_window_rect, get_monitor_work_rect, STABILIZE_ATTEMPTS, STABILIZE_INTERVAL_MS};

/// Bounding rect of the console window.
pub fn get_console_rect() -> Option<RECT> {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = get_console_hwnd()?;
        get_console_window_rect(hwnd)
    }
    #[cfg(not(target_os = "windows"))]
    None
}

pub fn get_window_rect() -> Option<RECT> {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = get_console_hwnd()?;
        get_console_window_rect(hwnd)
    }
    #[cfg(not(target_os = "windows"))]
    None
}

pub fn set_window_pos(x: i32, y: i32) {
    #[cfg(target_os = "windows")]
    unsafe {
        if let Some(hwnd) = get_console_hwnd() {
            windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                x,
                y,
                0,
                0,
                windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                    | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                    | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
            );
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (x, y);
    }
}

/// Center the console window on the primary display or active monitor.
pub fn center_console_window() {
    #[cfg(target_os = "windows")]
    unsafe {
        #[cfg(feature = "sys-info")]
        {
            let (_, terminal) = crate::sys_info::query_shell_and_terminal();
            if terminal != "Windows Console Host" {
                return;
            }
        }

        let hwnd = match get_console_hwnd() {
            Some(h) => h,
            None => return,
        };

        if windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible(hwnd) == 0 {
            return;
        }

        // Wait up to 200ms for the window size to stabilize after a resize
        let mut last_width = 0;
        let mut last_height = 0;
        let mut stable_count = 0;
        for _ in 0..STABILIZE_ATTEMPTS {
            if let Some(rect) = get_console_window_rect(hwnd) {
                let w = rect.right - rect.left;
                let h = rect.bottom - rect.top;
                if w == last_width && h == last_height && w > 0 && h > 0 {
                    stable_count += 1;
                    if stable_count >= 2 {
                        break;
                    }
                } else {
                    stable_count = 0;
                    last_width = w;
                    last_height = h;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(STABILIZE_INTERVAL_MS));
        }

        if let Some(rect) = get_console_window_rect(hwnd) {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            if let Some(work_rect) = get_monitor_work_rect(hwnd) {
                let monitor_w = work_rect.right - work_rect.left;
                let monitor_h = work_rect.bottom - work_rect.top;

                let x = work_rect.left + (monitor_w - width) / 2;
                let y = work_rect.top + (monitor_h - height) / 2;

                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    x,
                    y,
                    width,
                    height,
                    windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                );
            }
        }
    }
}

pub fn query_cursor_pos() -> Option<(i32, i32)> {
    #[cfg(target_os = "windows")]
    unsafe {
        let mut pt = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        if windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt) != 0 {
            Some((pt.x, pt.y))
        } else {
            None
        }
    }
    #[cfg(not(target_os = "windows"))]
    None
}
