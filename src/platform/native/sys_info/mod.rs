//! Host system information and theme querying utilities.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment - Native) + Role (System Software).
//! For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/rCommon/ARCHITECTURE.md).
//! Cross-platform with native features and platform-specific stubs.

#![allow(dead_code)]

pub use crate::platform::{PowerStatus, SystemBiosInfo, DiskDriveInfo, NetworkAdapterInfo, PlatformProvider};

type CachedAccent = (std::time::Instant, (u8, u8, u8));
type CachedBool = (std::time::Instant, bool);
type CachedString = (std::time::Instant, String);
type CachedTheme = (std::time::Instant, SystemTheme);
type CachedPower = (std::time::Instant, Option<PowerStatus>);

#[cfg(all(target_os = "windows", feature = "sys-info"))]
pub mod windows;

#[cfg(all(target_os = "linux", feature = "sys-info"))]
pub mod linux;

pub mod fallback;
pub mod glyphs;

#[cfg(all(target_os = "windows", not(feature = "sys-info")))]
use fallback as windows;

#[cfg(all(target_os = "linux", not(feature = "sys-info")))]
use fallback as linux;



pub mod providers;

#[cfg(target_os = "windows")]
pub use providers::WindowsPlatform;

#[cfg(target_os = "linux")]
pub use providers::LinuxPlatform;

#[cfg(all(
    not(any(target_os = "windows", target_os = "linux")),
    not(target_arch = "wasm32"),
    not(any(target_os = "android", target_os = "ios")),
    not(any(target_os = "none", target_os = "uefi"))
))]
pub use providers::FallbackPlatform;

#[cfg(feature = "widgets")]
pub fn get_dwm_accent_color() -> ratatui::style::Color {
    #[cfg(target_os = "windows")]
    {
        windows::get_dwm_accent_color()
    }
    #[cfg(not(target_os = "windows"))]
    {
        ratatui::style::Color::Cyan
    }
}

pub fn get_system_screen_resolution() -> (i32, i32) {
    crate::platform::CurrentPlatform::get_system_screen_resolution()
}

pub fn get_console_window_dpi() -> u32 {
    crate::platform::CurrentPlatform::get_console_window_dpi()
}

pub fn query_accent_color() -> (u8, u8, u8) {
    static CACHE: std::sync::Mutex<Option<CachedAccent>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(1000) {
            return *val;
        }
    }
    let val = crate::platform::CurrentPlatform::query_accent_color();
    *lock = Some((std::time::Instant::now(), val));
    val
}

pub fn query_high_contrast() -> bool {
    static CACHE: std::sync::Mutex<Option<CachedBool>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(1000) {
            return *val;
        }
    }
    let val = crate::platform::CurrentPlatform::query_high_contrast();
    *lock = Some((std::time::Instant::now(), val));
    val
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemTheme {
    pub is_dark_mode: bool,
    pub is_high_contrast: bool,
    pub accent_color: (u8, u8, u8),
}

/// Query the combined system theme settings (dark mode, high contrast, accent color).
pub fn query_system_theme() -> SystemTheme {
    static CACHE: std::sync::Mutex<Option<CachedTheme>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(500) {
            return *val;
        }
    }
    let is_dark_mode = query_dark_mode();
    let is_high_contrast = query_high_contrast();
    let accent_color = query_accent_color();
    let val = SystemTheme {
        is_dark_mode,
        is_high_contrast,
        accent_color,
    };
    *lock = Some((std::time::Instant::now(), val));
    val
}

pub fn query_os_version() -> String {
    static CACHE: std::sync::Mutex<Option<CachedString>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(10000) {
            return val.clone();
        }
    }
    let val = crate::platform::CurrentPlatform::query_os_version();
    *lock = Some((std::time::Instant::now(), val.clone()));
    val
}

pub fn query_dark_mode() -> bool {
    static CACHE: std::sync::Mutex<Option<CachedBool>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(500) {
            return *val;
        }
    }
    let val = crate::platform::CurrentPlatform::query_dark_mode();
    *lock = Some((std::time::Instant::now(), val));
    val
}

pub fn query_power_status() -> Option<PowerStatus> {
    static CACHE: std::sync::Mutex<Option<CachedPower>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(1000) {
            return val.clone();
        }
    }
    let val = crate::platform::CurrentPlatform::query_power_status();
    *lock = Some((std::time::Instant::now(), val.clone()));
    val
}

pub fn query_bios_info() -> Option<SystemBiosInfo> {
    static CACHE: std::sync::Mutex<Option<(std::time::Instant, Option<SystemBiosInfo>)>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(10000) {
            return val.clone();
        }
    }
    let val = crate::platform::CurrentPlatform::query_bios_info();
    *lock = Some((std::time::Instant::now(), val.clone()));
    val
}

pub fn query_shell_and_terminal() -> (String, String) {
    static SHELL_AND_TERM: std::sync::OnceLock<(String, String)> = std::sync::OnceLock::new();
    SHELL_AND_TERM
        .get_or_init(crate::platform::CurrentPlatform::query_shell_and_terminal)
        .clone()
}

pub use glyphs::GlyphMap;

pub fn query_disk_drives() -> Vec<DiskDriveInfo> {
    static CACHE: std::sync::Mutex<Option<(std::time::Instant, Vec<DiskDriveInfo>)>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(2000) {
            return val.clone();
        }
    }
    let val = crate::platform::CurrentPlatform::query_disk_drives();
    *lock = Some((std::time::Instant::now(), val.clone()));
    val
}

pub fn query_gpu_names() -> Vec<String> {
    static CACHE: std::sync::Mutex<Option<(std::time::Instant, Vec<String>)>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(5000) {
            return val.clone();
        }
    }
    let val = crate::platform::CurrentPlatform::query_gpu_names();
    *lock = Some((std::time::Instant::now(), val.clone()));
    val
}

pub fn query_network_adapters() -> Vec<NetworkAdapterInfo> {
    static CACHE: std::sync::Mutex<Option<(std::time::Instant, Vec<NetworkAdapterInfo>)>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(2000) {
            return val.clone();
        }
    }
    let val = crate::platform::CurrentPlatform::query_network_adapters();
    *lock = Some((std::time::Instant::now(), val.clone()));
    val
}

/// Collects all unique local IP addresses from network adapters (LAN + any others).
pub fn query_all_local_ips() -> Vec<String> {
    let mut ips = Vec::new();
    for adapter in query_network_adapters() {
        for ip in adapter.ip_addresses {
            if !ips.contains(&ip) {
                ips.push(ip);
            }
        }
    }
    if ips.is_empty() {
        if let Some(ip) = query_local_ip() {
            ips.push(ip);
        }
    }
    ips
}

pub fn query_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

// Definition lives in core.rs (single source of truth) so changes for one
// use-case cannot accidentally affect others.
pub use crate::core::DashboardInfo;

/// Returns a populated DashboardInfo using existing sys-info queries + sensible defaults.
/// Apps and effects can call this periodically for live data.
pub fn get_dashboard_info() -> DashboardInfo {
    static CACHE: std::sync::Mutex<Option<(std::time::Instant, DashboardInfo)>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(250) {
            return val.clone();
        }
    }
    let val = get_dashboard_info_uncached();
    *lock = Some((std::time::Instant::now(), val.clone()));
    val
}

fn get_dashboard_info_uncached() -> DashboardInfo {
    let os = query_os_version();
    let logo_text = if os.contains("Windows 11") || os.contains("WIN11") {
        "WIN11".to_string()
    } else if os.to_lowercase().contains("linux") {
        "LINUX".to_string()
    } else {
        "SYS".to_string()
    };

    #[allow(unused_mut)]
    let mut kernel = "unknown".to_string();
    #[allow(unused_mut)]
    let mut hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| crate::core::UNKNOWN_HOST.to_string());
    #[allow(unused_mut)]
    let mut cpu = "CPU".to_string();
    #[allow(unused_mut, unused_assignments)]
    let mut uptime_secs = 0;
    #[allow(unused_mut)]
    let mut mem_used_mb = 0;
    #[allow(unused_mut)]
    let mut mem_total_mb = 0;
    #[allow(unused_mut)]
    let mut mem_used_pct = 0.0;

    #[cfg(feature = "sys-info")]
    {
        use sysinfo::System;
        static SYSTEM: std::sync::Mutex<Option<System>> = std::sync::Mutex::new(None);
        let mut lock = SYSTEM.lock().unwrap();
        if lock.is_none() {
            *lock = Some(System::new());
        }
        let sys = lock.as_mut().unwrap();
        sys.refresh_cpu();
        sys.refresh_memory();
        
        if let Some(host) = System::host_name() {
            hostname = host;
        }
        if let Some(kv) = System::kernel_version() {
            kernel = kv;
        }
        
        let brand = sys.global_cpu_info().brand().trim().to_string();
        if !brand.is_empty() {
            cpu = brand;
        } else if !sys.cpus().is_empty() {
            cpu = sys.cpus()[0].brand().trim().to_string();
        }

        uptime_secs = System::uptime();

        let total = sys.total_memory();
        let used = sys.used_memory();
        if total > 0 {
            mem_total_mb = total / (1024 * 1024);
            mem_used_mb = used / (1024 * 1024);
            mem_used_pct = (used as f32 / total as f32) * 100.0;
        }
    }

    #[cfg(not(feature = "sys-info"))]
    {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
                kernel = content.trim().to_string();
            }
            if let Ok(content) = std::fs::read_to_string("/proc/sys/kernel/hostname") {
                hostname = content.trim().to_string();
            }
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    if line.starts_with("model name") {
                        if let Some(brand) = line.split(':').nth(1) {
                            cpu = brand.trim().to_string();
                            break;
                        }
                    }
                }
            }
            if let Ok(content) = std::fs::read_to_string("/proc/uptime") {
                if let Some(first) = content.split_whitespace().next() {
                    if let Ok(val) = first.parse::<f32>() {
                        uptime_secs = val as u64;
                    }
                }
            }
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                let mut mem_total_kb = 0u64;
                let mut mem_free_kb = 0u64;
                let mut mem_available_kb = 0u64;
                for line in content.lines() {
                    if line.starts_with("MemTotal:") {
                        mem_total_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    } else if line.starts_with("MemFree:") {
                        mem_free_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    } else if line.starts_with("MemAvailable:") {
                        mem_available_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    }
                }
                if mem_total_kb > 0 {
                    let available = if mem_available_kb > 0 { mem_available_kb } else { mem_free_kb };
                    let used_kb = mem_total_kb.saturating_sub(available);
                    mem_total_mb = mem_total_kb / 1024;
                    mem_used_mb = used_kb / 1024;
                    mem_used_pct = (used_kb as f32 / mem_total_kb as f32) * 100.0;
                }
            }
        }
    }

    let power = query_power_status().unwrap_or_default();
    let power_status = if power.ac_online {
        "AC".to_string()
    } else {
        format!("{}%", power.battery_percent)
    };

    let disks = query_disk_drives();
    let disk_summary = if let Some(d) = disks.first() {
        format!("{} ~{}G free", d.path, d.free_bytes / (1024 * 1024 * 1024))
    } else {
        "disks".to_string()
    };

    let gpu_list = query_gpu_names();
    let gpus = if gpu_list.is_empty() {
        "GPU(s)".to_string()
    } else {
        gpu_list.join(", ")
    };

    let monitor_count = super::monitors::get_all_monitors().len();
    let monitors = format!("{} monitor(s)", monitor_count);

    DashboardInfo {
        os,
        logo_text,
        kernel,
        hostname,
        cpu,
        uptime_secs,
        mem_used_mb,
        mem_total_mb,
        mem_used_pct,
        power_status,
        disk_summary,
        gpus,
        monitors,
    }
}

pub use crate::core::SystemInfo;

/// Returns rich live system info. Cross-platform (Windows/Linux/other stubs).
/// Classification: Platform (Native) + Core (neutral data).
pub fn get_system_info() -> SystemInfo {
    let dashboard = get_dashboard_info();
    SystemInfo {
        os: dashboard.os,
        logo_text: dashboard.logo_text,
        kernel: dashboard.kernel,
        hostname: dashboard.hostname,
        cpu: dashboard.cpu,
        uptime_secs: dashboard.uptime_secs,
        mem_used_mb: dashboard.mem_used_mb,
        mem_total_mb: dashboard.mem_total_mb,
        mem_used_pct: dashboard.mem_used_pct,
        power_status: dashboard.power_status,
        disk_summary: dashboard.disk_summary,
        gpus: dashboard.gpus,
        monitors: dashboard.monitors,
    }
}

pub use super::monitors::get_all_monitors;
pub use super::monitors::get_monitors_summary;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_ip() {
        let ip = query_local_ip();
        println!("Local IP: {:?}", ip);
    }

    #[test]
    fn test_sys_info_stubs() {
        let res = get_system_screen_resolution();
        assert!(res.0 > 0 && res.1 > 0);
        let dpi = get_console_window_dpi();
        assert!(dpi > 0);
    }

    #[test]
    fn test_dashboard_info() {
        let info = get_dashboard_info();
        assert!(!info.os.is_empty());
    }
}
