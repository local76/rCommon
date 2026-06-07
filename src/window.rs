#![allow(dead_code)]

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct MONITORINFO {
    pub cbSize: u32,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: u32,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct COORD {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct SMALL_RECT {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct CONSOLE_SELECTION_INFO {
    pub dwFlags: u32,
    pub dwSelectionAnchor: COORD,
    pub srSelection: SMALL_RECT,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

/// Bounding rect of the console window.
pub fn get_console_rect() -> Option<RECT> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() };
        if hwnd.is_null() {
            return None;
        }
        let mut r = RECT::default();
        let ok = unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(
                hwnd,
                &mut r as *mut RECT as *mut windows_sys::Win32::Foundation::RECT,
            )
        };
        if ok != 0 {
            Some(r)
        } else {
            None
        }
    }
    #[cfg(not(target_os = "windows"))]
    None
}

// Re-export SingleInstanceGuard from guard module to preserve API compatibility
pub use crate::guard::SingleInstanceGuard;

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
            let hwnd = unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() };
            if hwnd.is_null() {
                return BorderlessConsole {
                    hwnd: std::ptr::null_mut(),
                    original_style: 0,
                    original_rect: RECT::default(),
                    active: false,
                };
            }

            let original_style = unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                    hwnd,
                    -16, // GWL_STYLE = -16
                )
            };
            let mut original_rect = RECT::default();
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(
                    hwnd,
                    &mut original_rect as *mut RECT as *mut windows_sys::Win32::Foundation::RECT,
                );
            }

            // Strip border decorations: WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU
            let style_mask = 0x00C00000 | 0x00040000 | 0x00020000 | 0x00010000 | 0x00080000;
            let new_style = original_style & !(style_mask as isize);
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                    hwnd,
                    -16, // GWL_STYLE = -16
                    new_style,
                );
            }

            let dpi = unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForWindow(hwnd) };
            let scale = dpi as f32 / 96.0;
            let width = (900.0 * scale) as i32;
            let height = (900.0 * scale) as i32;

            let mut x = 100;
            let mut y = 100;
            let h_monitor = unsafe {
                windows_sys::Win32::Graphics::Gdi::MonitorFromWindow(
                    hwnd,
                    windows_sys::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST,
                )
            };
            if !h_monitor.is_null() {
                let mut mi = windows_sys::Win32::Graphics::Gdi::MONITORINFO {
                    cbSize: std::mem::size_of::<windows_sys::Win32::Graphics::Gdi::MONITORINFO>() as u32,
                    rcMonitor: windows_sys::Win32::Foundation::RECT { left: 0, top: 0, right: 0, bottom: 0 },
                    rcWork: windows_sys::Win32::Foundation::RECT { left: 0, top: 0, right: 0, bottom: 0 },
                    dwFlags: 0,
                };
                if unsafe {
                    windows_sys::Win32::Graphics::Gdi::GetMonitorInfoW(h_monitor, &mut mi as *mut _)
                } != 0
                {
                    let monitor_w = mi.rcWork.right - mi.rcWork.left;
                    let monitor_h = mi.rcWork.bottom - mi.rcWork.top;
                    x = mi.rcWork.left + (monitor_w - width) / 2;
                    y = mi.rcWork.top + (monitor_h - height) / 2;
                }
            }

            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    x,
                    y,
                    width,
                    height,
                    windows_sys::Win32::UI::WindowsAndMessaging::SWP_FRAMECHANGED
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
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
                    -16, // GWL_STYLE = -16
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
                    windows_sys::Win32::UI::WindowsAndMessaging::SWP_FRAMECHANGED
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                );
            }
        }
    }
}

/// Center the console window on the primary display or active monitor.
pub fn center_console_window() {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = windows_sys::Win32::System::Console::GetConsoleWindow();
        if hwnd.is_null() {
            return;
        }

        // Wait up to 200ms for the window size to stabilize after a resize
        let mut last_width = 0;
        let mut last_height = 0;
        let mut stable_count = 0;
        for _ in 0..20 {
            let mut rect = RECT::default();
            if windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(
                hwnd,
                &mut rect as *mut RECT as *mut windows_sys::Win32::Foundation::RECT,
            ) != 0
            {
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
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let mut rect = RECT::default();
        if windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(
            hwnd,
            &mut rect as *mut RECT as *mut windows_sys::Win32::Foundation::RECT,
        ) != 0
        {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let h_monitor = windows_sys::Win32::Graphics::Gdi::MonitorFromWindow(
                hwnd,
                windows_sys::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST,
            );
            if !h_monitor.is_null() {
                let mut mi = windows_sys::Win32::Graphics::Gdi::MONITORINFO {
                    cbSize: std::mem::size_of::<windows_sys::Win32::Graphics::Gdi::MONITORINFO>() as u32,
                    rcMonitor: windows_sys::Win32::Foundation::RECT { left: 0, top: 0, right: 0, bottom: 0 },
                    rcWork: windows_sys::Win32::Foundation::RECT { left: 0, top: 0, right: 0, bottom: 0 },
                    dwFlags: 0,
                };
                if windows_sys::Win32::Graphics::Gdi::GetMonitorInfoW(h_monitor, &mut mi as *mut _) != 0 {
                    let monitor_w = mi.rcWork.right - mi.rcWork.left;
                    let monitor_h = mi.rcWork.bottom - mi.rcWork.top;

                    let x = mi.rcWork.left + (monitor_w - width) / 2;
                    let y = mi.rcWork.top + (monitor_h - height) / 2;

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

pub fn get_window_rect() -> Option<RECT> {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = windows_sys::Win32::System::Console::GetConsoleWindow();
        if !hwnd.is_null() {
            let mut rect = RECT::default();
            if windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(
                hwnd,
                &mut rect as *mut RECT as *mut windows_sys::Win32::Foundation::RECT,
            ) != 0
            {
                return Some(rect);
            }
        }
    }
    None
}

pub fn set_window_pos(x: i32, y: i32) {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = windows_sys::Win32::System::Console::GetConsoleWindow();
        if !hwnd.is_null() {
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
                Some(buf[..len as usize].to_vec())
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

/// If the application is running in a pseudoconsole (like Windows Terminal) and we want it
/// to run as a standalone styled window, relaunch it inside conhost.exe.
pub fn relaunch_in_conhost_if_needed() {
    #[cfg(target_os = "windows")]
    {
        // 1. Check if we have the --relaunched flag to prevent any potential loops
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|arg| arg == "--relaunched") {
            return;
        }

        // 2. Check if there are arguments that request stdout/diagnostic mode
        for arg in &args {
            let lower = arg.to_lowercase();
            if lower == "--help"
                || lower == "-h"
                || lower == "help"
                || lower == "--version"
                || lower == "-v"
                || lower == "version"
                || lower == "install"
                || lower == "--install"
            {
                return;
            }
        }

        // 3. Detect if we are in conhost or a pseudoconsole (like Windows Terminal)
        let (_, terminal) = crate::sys_info::query_shell_and_terminal();
        let is_conhost = terminal == "Windows Console Host" && {
            let hwnd = unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() };
            if hwnd.is_null() {
                false
            } else {
                let mut rect = RECT::default();
                let ok = unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(
                        hwnd,
                        &mut rect as *mut RECT as *mut windows_sys::Win32::Foundation::RECT,
                    )
                };
                let style = unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                        hwnd,
                        -16, // GWL_STYLE = -16
                    )
                };
                ok != 0 && (rect.right - rect.left) > 0 && style != 0
            }
        };

        if !is_conhost {
            // Relaunch in conhost.exe directly to avoid cmd.exe quoting/escaping bugs
            if let Ok(current_exe) = std::env::current_exe() {
                let mut con_args = vec![current_exe.to_string_lossy().to_string()];
                // Pass all original args, plus the --relaunched flag
                for arg in args.into_iter().skip(1) {
                    con_args.push(arg);
                }
                con_args.push("--relaunched".to_string());

                let _ = std::process::Command::new("conhost.exe")
                    .args(&con_args)
                    .spawn();
                std::process::exit(0);
            }
        }
    }
}
