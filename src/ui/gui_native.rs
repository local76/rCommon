//! GUI Native/OS helpers.
//!
//! Part of the Interface (Presentation Layer) - GUI: Native/OS.
//! Basic cross-platform native dialog and message helpers.
//! **Taxonomy Classification**: Interface (GUI Native).
//!
//! For full native, consumers can extend with platform crates.
//!
//! **Feature Stub**: This is a fallback placeholder implementation providing safe, parameter-equivalent default values when compiled for target platforms where the native implementation is unavailable.

#[cfg(all(windows, feature = "windows-sys"))]
use std::ffi::OsStr;
#[cfg(all(windows, feature = "windows-sys"))]
use std::os::windows::ffi::OsStrExt;

/// Simple message box dialog (Win32 MessageBox on Windows, console print on others).
#[cfg(all(windows, feature = "windows-sys"))]
pub fn show_message_box(title: &str, message: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};
    let title_w: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
    let msg_w: Vec<u16> = OsStr::new(message).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(std::ptr::null_mut(), msg_w.as_ptr(), title_w.as_ptr(), MB_OK | MB_ICONINFORMATION);
    }
}

#[cfg(any(not(windows), not(feature = "windows-sys")))]
pub fn show_message_box(title: &str, message: &str) {
    println!("[INFO] {}: {}", title, message);
}

/// Native error dialog box.
#[cfg(all(windows, feature = "windows-sys"))]
pub fn show_error_box(title: &str, message: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let title_w: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
    let msg_w: Vec<u16> = OsStr::new(message).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(std::ptr::null_mut(), msg_w.as_ptr(), title_w.as_ptr(), MB_OK | MB_ICONERROR);
    }
}

#[cfg(any(not(windows), not(feature = "windows-sys")))]
pub fn show_error_box(title: &str, message: &str) {
    eprintln!("[ERROR] {}: {}", title, message);
}

/// Native warning dialog box.
#[cfg(all(windows, feature = "windows-sys"))]
pub fn show_warning_box(title: &str, message: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONWARNING, MB_OK};
    let title_w: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
    let msg_w: Vec<u16> = OsStr::new(message).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(std::ptr::null_mut(), msg_w.as_ptr(), title_w.as_ptr(), MB_OK | MB_ICONWARNING);
    }
}

#[cfg(any(not(windows), not(feature = "windows-sys")))]
pub fn show_warning_box(title: &str, message: &str) {
    eprintln!("[WARNING] {}: {}", title, message);
}

/// Native confirmation dialog (Yes/No prompt).
/// Returns true if the user confirmed (clicked Yes), false otherwise.
#[cfg(all(windows, feature = "windows-sys"))]
pub fn show_confirm_dialog(title: &str, message: &str) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONQUESTION, MB_YESNO, IDYES};
    let title_w: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
    let msg_w: Vec<u16> = OsStr::new(message).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        let result = MessageBoxW(
            std::ptr::null_mut(),
            msg_w.as_ptr(),
            title_w.as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        );
        result == IDYES
    }
}

#[cfg(any(not(windows), not(feature = "windows-sys")))]
pub fn show_confirm_dialog(_title: &str, message: &str) -> bool {
    use std::io::{self, Write};
    print!("{} [y/N]: ", message);
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim().to_lowercase();
        trimmed == "y" || trimmed == "yes"
    } else {
        false
    }
}
