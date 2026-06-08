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
/// Falls back to the rCommon `UNKNOWN_HOST` constant on failure.
pub fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| UNKNOWN_HOST.to_string())
}

/// Returns `"{username}@{hostname}"` formatted for the title banner.
pub fn user_host() -> String {
    format!("{}@{}", username(), hostname())
}

/// Returns the OS version string (delegates to rcommon's cached query).
pub fn os_str() -> String {
    query_os_version()
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
