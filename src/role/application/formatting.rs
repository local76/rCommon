//! Formatting and info helpers from helm etc.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).
//!
//! Reusable system formatting utilities for display in terminal/TUI dashboards.
//!
//! # Examples
//!
//! ```
//! use library::role::application::formatting;
//!
//! let host = formatting::get_host_info();
//! println!("Host name: {}", host);
//!
//! let uptime = formatting::get_formatted_uptime();
//! println!("Uptime: {}", uptime);
//! ```

#[cfg(feature = "sys-info")]
use sysinfo::System;

/// Get host/computer name.
pub fn get_host_info() -> String {
    #[cfg(feature = "sys-info")]
    {
        if let Some(host) = System::host_name() {
            return host;
        }
    }
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| crate::core::UNKNOWN_HOST.to_string())
}

/// Get CPU info.
#[cfg(feature = "sys-info")]
pub fn get_cpu_info(sys: &System) -> String {
    let brand = sys.global_cpu_info().brand();
    if !brand.is_empty() {
        brand.trim().to_string()
    } else if !sys.cpus().is_empty() {
        sys.cpus()[0].brand().trim().to_string()
    } else {
        "Unknown CPU".to_string()
    }
}

/// Get formatted uptime (e.g. "1d 2h 45m" or "2h 45m").
///
/// # Examples
///
/// ```
/// use library::role::application::formatting;
///
/// let uptime = formatting::get_formatted_uptime();
/// assert!(uptime.contains('h') || uptime.contains('d'));
/// ```
pub fn get_formatted_uptime() -> String {
    let uptime_secs = crate::sys_info::get_system_info().uptime_secs;
    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let minutes = (uptime_secs % 3600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else {
        format!("{}h {}m", hours, minutes)
    }
}

/// Get battery status summary text.
///
/// # Examples
///
/// ```
/// use library::role::application::formatting;
///
/// let battery = formatting::get_battery_info();
/// println!("Battery status: {}", battery);
/// ```
pub fn get_battery_info() -> String {
    if let Some(power) = crate::sys_info::query_power_status() {
        format!("{}% ({})", power.battery_percent, if power.ac_online { "AC" } else { "Battery" })
    } else {
        "N/A".to_string()
    }
}

/// Get formatted memory usage (used / total in MB).
#[cfg(feature = "sys-info")]
pub fn get_memory_info(sys: &System) -> String {
    let total = sys.total_memory();
    let used = sys.used_memory();
    if total > 0 {
        let total_mb = total / (1024 * 1024);
        let used_mb = used / (1024 * 1024);
        let pct = (used as f64 / total as f64 * 100.0).round() as u64;
        format!("{} / {} MB ({}%)", used_mb, total_mb, pct)
    } else {
        "0 / 0 MB".to_string()
    }
}

/// Get formatted disk information summary for all drives.
///
/// # Examples
///
/// ```
/// use library::role::application::formatting;
///
/// let disks = formatting::get_disks_info();
/// println!("Drives: {}", disks);
/// ```
pub fn get_disks_info() -> String {
    let drives = crate::sys_info::query_disk_drives();
    let mut parts = Vec::new();
    for d in drives {
        let total_gb = d.total_bytes / (1024 * 1024 * 1024);
        let free_gb = d.free_bytes / (1024 * 1024 * 1024);
        parts.push(format!("{}: {}/{} GB free", d.path, free_gb, total_gb));
    }
    if parts.is_empty() {
        "N/A".to_string()
    } else {
        parts.join(", ")
    }
}

/// Get formatted GPU list.
pub fn get_gpu_names() -> String {
    let gpus = crate::sys_info::query_gpu_names();
    if gpus.is_empty() {
        "Unknown GPU".to_string()
    } else {
        gpus.join(", ")
    }
}

/// Detect host shell and active terminal type.
pub fn detect_shell_and_terminal() -> (String, String) {
    crate::sys_info::query_shell_and_terminal()
}

/// Helper to convert a hex color string (e.g. "#D41020" or "D41020") to RGB (u8, u8, u8).
pub fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(245);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        (r, g, b)
    } else {
        (0, 245, 255) // fallback default accent color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(hex_to_rgb("#ff0000"), (255, 0, 0));
        assert_eq!(hex_to_rgb("00ff00"), (0, 255, 0));
        assert_eq!(hex_to_rgb("invalid"), (0, 245, 255));
    }

    #[test]
    fn test_formatting_helpers() {
        let host = get_host_info();
        assert!(!host.is_empty());

        #[cfg(feature = "sys-info")]
        {
            use sysinfo::System;
            let mut sys = System::new();
            sys.refresh_cpu();
            sys.refresh_memory();
            
            let cpu = get_cpu_info(&sys);
            assert!(!cpu.is_empty());
            
            let memory = get_memory_info(&sys);
            assert!(!memory.is_empty());
        }

        let uptime = get_formatted_uptime();
        assert!(!uptime.is_empty());

        let battery = get_battery_info();
        assert!(!battery.is_empty());

        let disks = get_disks_info();
        assert!(!disks.is_empty());

        let gpu = get_gpu_names();
        assert!(!gpu.is_empty());

        let (shell, term) = detect_shell_and_terminal();
        assert!(!shell.is_empty());
        assert!(!term.is_empty());
    }
}