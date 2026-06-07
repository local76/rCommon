use std::io;
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};
#[cfg(target_os = "linux")]
use std::io::Write;

/// Set the system clipboard text using native Win32 APIs (on Windows), command-line tools (on Linux), or stubs.
pub fn copy_text_to_clipboard(text: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        if windows_sys::Win32::System::DataExchange::OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err(io::Error::last_os_error());
        }
        if windows_sys::Win32::System::DataExchange::EmptyClipboard() == 0 {
            let _ = windows_sys::Win32::System::DataExchange::CloseClipboard();
            return Err(io::Error::last_os_error());
        }

        let text_w: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let len = text_w.len() * 2;
        let h_mem = windows_sys::Win32::System::Memory::GlobalAlloc(
            windows_sys::Win32::System::Memory::GMEM_MOVEABLE,
            len,
        );
        if h_mem.is_null() {
            let _ = windows_sys::Win32::System::DataExchange::CloseClipboard();
            return Err(io::Error::last_os_error());
        }

        let ptr = windows_sys::Win32::System::Memory::GlobalLock(h_mem);
        if ptr.is_null() {
            let _ = windows_sys::Win32::Foundation::GlobalFree(h_mem);
            let _ = windows_sys::Win32::System::DataExchange::CloseClipboard();
            return Err(io::Error::last_os_error());
        }

        std::ptr::copy_nonoverlapping(text_w.as_ptr(), ptr as *mut u16, text_w.len());
        windows_sys::Win32::System::Memory::GlobalUnlock(h_mem);

        if windows_sys::Win32::System::DataExchange::SetClipboardData(
            windows_sys::Win32::System::Ole::CF_UNICODETEXT as u32,
            h_mem,
        ).is_null()
        {
            let _ = windows_sys::Win32::Foundation::GlobalFree(h_mem);
            let _ = windows_sys::Win32::System::DataExchange::CloseClipboard();
            return Err(io::Error::last_os_error());
        }

        windows_sys::Win32::System::DataExchange::CloseClipboard();
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // 1. Try wl-copy (Wayland)
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        // 2. Fallback to xclip (X11)
        if let Ok(mut child) = Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        // 3. Fallback to xsel (X11)
        if let Ok(mut child) = Command::new("xsel")
            .args(&["--clipboard", "--input"])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No supported clipboard utility found (wl-copy, xclip, or xsel)",
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = text;
        Ok(())
    }
}
