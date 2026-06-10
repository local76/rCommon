//! Windows-specific console and window management.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

use std::io;
use crate::apps::window::types::RECT;

pub(crate) const BORDERLESS_DEFAULT_SIZE: f32 = 900.0;
pub(crate) const STABILIZE_ATTEMPTS: u32 = 20;
pub(crate) const STABILIZE_INTERVAL_MS: u64 = 10;
pub(crate) const SWP_FRAMECHANGED_NOOP: u32 =
    windows_sys::Win32::UI::WindowsAndMessaging::SWP_FRAMECHANGED
        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE;
pub(crate) const GWL_STYLE: i32 = windows_sys::Win32::UI::WindowsAndMessaging::GWL_STYLE;
pub(crate) const WS_DECORATIONS: u32 = windows_sys::Win32::UI::WindowsAndMessaging::WS_CAPTION
    | windows_sys::Win32::UI::WindowsAndMessaging::WS_THICKFRAME
    | windows_sys::Win32::UI::WindowsAndMessaging::WS_MINIMIZEBOX
    | windows_sys::Win32::UI::WindowsAndMessaging::WS_MAXIMIZEBOX
    | windows_sys::Win32::UI::WindowsAndMessaging::WS_SYSMENU;
pub(crate) const SW_HIDE: i32 = windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

pub(crate) unsafe fn get_console_hwnd() -> Option<*mut std::ffi::c_void> {
    let hwnd = unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() };
    if hwnd.is_null() {
        None
    } else {
        Some(hwnd)
    }
}

pub(crate) unsafe fn get_console_window_rect(hwnd: *mut std::ffi::c_void) -> Option<RECT> {
    let mut rect = RECT::default();
    let ok = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(
            hwnd,
            &mut rect as *mut RECT as *mut windows_sys::Win32::Foundation::RECT,
        )
    };
    if ok != 0 {
        Some(rect)
    } else {
        None
    }
}

pub(crate) unsafe fn get_monitor_work_rect(hwnd: *mut std::ffi::c_void) -> Option<RECT> {
    let h_monitor = unsafe {
        windows_sys::Win32::Graphics::Gdi::MonitorFromWindow(
            hwnd,
            windows_sys::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST,
        )
    };
    if h_monitor.is_null() {
        return None;
    }
    let mut mi = windows_sys::Win32::Graphics::Gdi::MONITORINFO {
        cbSize: std::mem::size_of::<windows_sys::Win32::Graphics::Gdi::MONITORINFO>() as u32,
        rcMonitor: windows_sys::Win32::Foundation::RECT { left: 0, top: 0, right: 0, bottom: 0 },
        rcWork: windows_sys::Win32::Foundation::RECT { left: 0, top: 0, right: 0, bottom: 0 },
        dwFlags: 0,
    };
    let ok = unsafe {
        windows_sys::Win32::Graphics::Gdi::GetMonitorInfoW(h_monitor, &mut mi as *mut _)
    };
    if ok != 0 {
        Some(RECT {
            left: mi.rcWork.left,
            top: mi.rcWork.top,
            right: mi.rcWork.right,
            bottom: mi.rcWork.bottom,
        })
    } else {
        None
    }
}

pub fn get_console_rect() -> Option<RECT> {
    unsafe {
        let hwnd = get_console_hwnd()?;
        get_console_window_rect(hwnd)
    }
}

pub fn get_window_rect() -> Option<RECT> {
    unsafe {
        let hwnd = get_console_hwnd()?;
        get_console_window_rect(hwnd)
    }
}

pub fn set_window_pos(x: i32, y: i32) {
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
}

pub fn center_console_window() {
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
    unsafe {
        let mut pt = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        if windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt) != 0 {
            Some((pt.x, pt.y))
        } else {
            None
        }
    }
}

pub fn should_relaunch_in_conhost() -> bool {
    // Detect if we are in conhost or a pseudoconsole (like Windows Terminal)
    #[cfg(feature = "sys-info")]
    {
        let (_, terminal) = crate::sys_info::query_shell_and_terminal();
        let is_conhost = terminal == "Windows Console Host" && unsafe {
            if let Some(hwnd) = get_console_hwnd() {
                if let Some(rect) = get_console_window_rect(hwnd) {
                    let style = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                        hwnd,
                        GWL_STYLE,
                    );
                    (rect.right - rect.left) > 0 && style != 0
                } else {
                    false
                }
            } else {
                false
            }
        };

        !is_conhost
    }
    #[cfg(not(feature = "sys-info"))]
    {
        false
    }
}

pub fn relaunch_in_conhost() -> io::Result<()> {
    let current_exe = std::env::current_exe()?;
    let args: Vec<String> = std::env::args().collect();
    let mut con_args = vec![current_exe.to_string_lossy().to_string()];
    con_args.extend(args.into_iter().skip(1));
    con_args.push("--relaunched".to_string());

    std::process::Command::new("conhost.exe")
        .args(&con_args)
        .spawn()?;
    Ok(())
}

#[allow(deprecated)]
pub fn relaunch_in_conhost_if_needed() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--relaunched") {
        return;
    }

    // Check if there are arguments that request stdout/diagnostic/help mode
    for arg in &args {
        if crate::core::is_help_arg(arg)
            || crate::core::is_version_arg(arg)
            || crate::core::is_doctor_arg(arg)
            || crate::core::is_install_arg(arg)
        {
            return;
        }
    }

    if should_relaunch_in_conhost() {
        if relaunch_in_conhost().is_ok() {
            std::process::exit(0);
        } else {
            eprintln!("Warning: Failed to relaunch in conhost.exe, continuing in current terminal.");
        }
    }
}

pub fn hide_console_at_startup() -> Option<*mut std::ffi::c_void> {
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

pub fn show_console_window() {
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

pub fn is_console_focused() -> bool {
    unsafe {
        let hwnd = match get_console_hwnd() {
            Some(h) => h,
            None => return false,
        };
        let fg = windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
        hwnd == fg
    }
}

pub struct BorderlessConsole {
    hwnd: *mut std::ffi::c_void,
    original_style: isize,
    original_rect: RECT,
    active: bool,
}

impl BorderlessConsole {
    pub fn enable() -> Self {
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

    pub fn enable_preserving_size() -> Self {
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
}

impl Drop for BorderlessConsole {
    fn drop(&mut self) {
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
}

impl Drop for ConsoleTitleGuard {
    fn drop(&mut self) {
        if let Some(ref title) = self.original_title {
            let mut title_null = title.clone();
            title_null.push(0);
            unsafe {
                windows_sys::Win32::System::Console::SetConsoleTitleW(title_null.as_ptr());
            }
        }
    }
}
