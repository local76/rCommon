//! Identity helpers (username, hostname, OS string) for TUI banner rendering.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).
//!
//! Cross-platform resolution of `$USERNAME`/`$USER`, `$COMPUTERNAME`/`$HOSTNAME`,
//! and the cached system OS version string. Used by every r* TUI's title banner.

use crate::core::UNKNOWN_HOST;
use crate::platform::native::sys_info::query_os_version;

/// Returns the current OS username (Windows: `$USERNAME`, POSIX: `$USER`).
pub fn username() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "user".to_string())
}

/// Returns the current hostname (Windows: `$COMPUTERNAME`, POSIX: `$HOSTNAME`).
/// Falls back to the library `UNKNOWN_HOST` constant on failure.
pub fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| UNKNOWN_HOST.to_string())
}

/// Returns `"{username}@{hostname}"` formatted for the title banner.
pub fn user_host() -> String {
    format!("{}@{}", username(), hostname())
}

/// Returns the OS version string (delegates to library's cached query).
pub fn os_str() -> String {
    query_os_version()
}

/// Returns the user's default shell (Windows: PowerShell v7.4 if `$PSModulePath`
/// is set, else `cmd.exe`; POSIX: `$SHELL` env var, default `/bin/bash`).
/// Used by the bounce dashboard's "Shell:" field.
pub fn shell_name() -> String {
    if cfg!(target_os = "windows") {
        if std::env::var("PSModulePath").is_ok() {
            "PowerShell v7.4".to_string()
        } else {
            "cmd.exe".to_string()
        }
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}

/// Returns the main monitor's refresh rate in Hz. Windows: queries the
/// device caps (VREFRESH = 116). Other platforms: returns 60 as a sane default.
/// Used by the bounce dashboard's "Display:" field.
pub fn refresh_rate_hz() -> i32 {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC};
        unsafe {
            let hdc = GetDC(std::ptr::null_mut());
            if !hdc.is_null() {
                let rate = GetDeviceCaps(hdc, 116);
                ReleaseDC(std::ptr::null_mut(), hdc);
                if rate <= 0 { 144 } else { rate }
            } else {
                144
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        60
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_host_format() {
        let uh = user_host();
        assert!(uh.contains('@'));
    }

    #[test]
    fn test_username_not_empty() {
        assert!(!username().is_empty());
    }

    #[test]
    fn test_hostname_not_empty() {
        assert!(!hostname().is_empty());
    }
}
