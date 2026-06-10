//! Conhost relaunch routine for Windows Terminal / pseudoconsole execution.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

#[cfg(target_os = "windows")]
use super::types::{get_console_hwnd, get_console_window_rect, GWL_STYLE};

/// Check if the application should be relaunched in conhost.exe.
///
/// This returns true if we are on Windows, we are NOT running under conhost (e.g., we are
/// running inside Windows Terminal or another pseudoconsole), and a standalone console
/// host is required for window styling.
pub fn should_relaunch_in_conhost() -> bool {
    #[cfg(target_os = "windows")]
    {
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
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Relaunches the current process inside conhost.exe.
///
/// This spawns `conhost.exe` running the same executable and argument list,
/// and returns the child handle or error. The caller should exit the current
/// process if this succeeds.
pub fn relaunch_in_conhost() -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
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
    #[cfg(not(target_os = "windows"))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Conhost relaunch is only supported on Windows",
        ))
    }
}

/// Deprecated compatibility wrapper that automatically checks arguments,
/// checks if relaunch is needed, spawns the child, and exits the process.
#[deprecated(
    since = "1.0.0",
    note = "Use should_relaunch_in_conhost and relaunch_in_conhost directly in main"
)]
pub fn relaunch_in_conhost_if_needed() {
    #[cfg(target_os = "windows")]
    {
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
}
