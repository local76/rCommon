//! Terminal styling, borders, and decor management.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

use super::types::RECT;

#[cfg(target_os = "windows")]
use super::types::{
    get_console_hwnd, get_console_window_rect, get_monitor_work_rect,
    SWP_FRAMECHANGED_NOOP, GWL_STYLE, WS_DECORATIONS, BORDERLESS_DEFAULT_SIZE,
};

/// Strips standard console headers/borders and centers window dynamically (DPI-aware).
pub struct BorderlessConsole {
    hwnd: *mut std::ffi::c_void,
    original_style: isize,
    original_rect: RECT,
    active: bool,
}

impl BorderlessConsole {
    pub fn enable() -> Self {
        #[cfg(target_os = "windows")]
        {
            #[cfg(feature = "sys-info")]
            {
            let (_, terminal) = crate::sys_info::query_shell_and_terminal();
            if terminal != "Windows Console Host" {
                return BorderlessConsole {
                    hwnd: std::ptr::null_mut(),
                    original_style: 0,
                    original_rect: RECT::default(),
                    active: false,
                };
            }
            }

            let hwnd = match unsafe { get_console_hwnd() } {
                Some(h) => h,
                None => {
                    return BorderlessConsole {
                        hwnd: std::ptr::null_mut(),
                        original_style: 0,
                        original_rect: RECT::default(),
                        active: false,
                    };
                }
            };

            let original_style = unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                    hwnd,
                    GWL_STYLE,
                )
            };
            let original_rect = unsafe { get_console_window_rect(hwnd) }.unwrap_or_default();

            // Strip border decorations
            let new_style = original_style & !(WS_DECORATIONS as isize);
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                    hwnd,
                    GWL_STYLE,
                    new_style,
                );
            }

            let dpi = unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForWindow(hwnd) };
            let scale = dpi as f32 / 96.0;
            let width = (BORDERLESS_DEFAULT_SIZE * scale) as i32;
            let height = (BORDERLESS_DEFAULT_SIZE * scale) as i32;

            let mut x = 100;
            let mut y = 100;
            if let Some(work_rect) = unsafe { get_monitor_work_rect(hwnd) } {
                let monitor_w = work_rect.right - work_rect.left;
                let monitor_h = work_rect.bottom - work_rect.top;
                x = work_rect.left + (monitor_w - width) / 2;
                y = work_rect.top + (monitor_h - height) / 2;
            }

            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    x,
                    y,
                    width,
                    height,
                    SWP_FRAMECHANGED_NOOP,
                );
            }

            BorderlessConsole {
                hwnd,
                original_style,
                original_rect,
                active: true,
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            BorderlessConsole {
                hwnd: std::ptr::null_mut(),
                original_style: 0,
                original_rect: RECT::default(),
                active: false,
            }
        }
    }

    pub fn enable_preserving_size() -> Self {
        #[cfg(target_os = "windows")]
        {
            #[cfg(feature = "sys-info")]
            {
                let (_, terminal) = crate::sys_info::query_shell_and_terminal();
                if terminal != "Windows Console Host" {
                    return BorderlessConsole {
                        hwnd: std::ptr::null_mut(),
                        original_style: 0,
                        original_rect: RECT::default(),
                        active: false,
                    };
                }
            }

            let hwnd = match unsafe { get_console_hwnd() } {
                Some(h) => h,
                None => {
                    return BorderlessConsole {
                        hwnd: std::ptr::null_mut(),
                        original_style: 0,
                        original_rect: RECT::default(),
                        active: false,
                    };
                }
            };

            let original_style = unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                    hwnd,
                    GWL_STYLE,
                )
            };
            let original_rect = unsafe { get_console_window_rect(hwnd) }.unwrap_or_default();

            // Strip border decorations
            let new_style = original_style & !(WS_DECORATIONS as isize);
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                    hwnd,
                    GWL_STYLE,
                    new_style,
                );
            }

            let width = original_rect.right - original_rect.left;
            let height = original_rect.bottom - original_rect.top;

            let mut x = 100;
            let mut y = 100;
            if let Some(work_rect) = unsafe { get_monitor_work_rect(hwnd) } {
                let monitor_w = work_rect.right - work_rect.left;
                let monitor_h = work_rect.bottom - work_rect.top;
                x = work_rect.left + (monitor_w - width) / 2;
                y = work_rect.top + (monitor_h - height) / 2;
            }

            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    x,
                    y,
                    width,
                    height,
                    SWP_FRAMECHANGED_NOOP,
                );
            }

            BorderlessConsole {
                hwnd,
                original_style,
                original_rect,
                active: true,
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            BorderlessConsole {
                hwnd: std::ptr::null_mut(),
                original_style: 0,
                original_rect: RECT::default(),
                active: false,
            }
        }
    }
}

impl Drop for BorderlessConsole {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if self.active && !self.hwnd.is_null() {
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                    self.hwnd,
                    GWL_STYLE,
                    self.original_style,
                );
                let width = self.original_rect.right - self.original_rect.left;
                let height = self.original_rect.bottom - self.original_rect.top;
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    self.hwnd,
                    std::ptr::null_mut(),
                    self.original_rect.left,
                    self.original_rect.top,
                    width,
                    height,
                    SWP_FRAMECHANGED_NOOP,
                );
            }
        }
    }
}

pub struct ConsoleTitleGuard {
    original_title: Option<Vec<u16>>,
}

impl ConsoleTitleGuard {
    pub fn new(new_title: &str) -> Self {
        #[cfg(target_os = "windows")]
        {
            let mut buf = [0u16; 512];
            let len = unsafe {
                windows_sys::Win32::System::Console::GetConsoleTitleW(
                    buf.as_mut_ptr(),
                    buf.len() as u32,
                )
            };
            let original_title = if len > 0 {
                let safe_len = (len as usize).min(buf.len());
                Some(buf[..safe_len].to_vec())
            } else {
                None
            };

            let title_w: Vec<u16> = new_title.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                windows_sys::Win32::System::Console::SetConsoleTitleW(title_w.as_ptr());
            }

            ConsoleTitleGuard { original_title }
        }
        #[cfg(target_os = "linux")]
        {
            use std::io::Write;
            // Push current title to terminal stack, then set new title
            print!("\x1b[22;2t\x1b]2;{}\x07", new_title);
            let _ = std::io::stdout().flush();
            ConsoleTitleGuard {
                original_title: None,
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            let _ = new_title;
            ConsoleTitleGuard {
                original_title: None,
            }
        }
    }
}

impl Drop for ConsoleTitleGuard {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if let Some(ref title) = self.original_title {
            let mut title_null = title.clone();
            title_null.push(0);
            unsafe {
                windows_sys::Win32::System::Console::SetConsoleTitleW(title_null.as_ptr());
            }
        }
        #[cfg(target_os = "linux")]
        {
            use std::io::Write;
            // Pop original title from terminal stack
            print!("\x1b[23;2t");
            let _ = std::io::stdout().flush();
        }
    }
}
