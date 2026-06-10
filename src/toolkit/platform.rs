//! Platform & Architecture (Deployment)
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment).
//!
//! How the software is packaged and where it is designed to run.
//!
//! ### Platform & Feature Stubs
//!
//! To support clean cross-platform compilation and predictable fallback behavior,
//! this codebase follows a unified design for non-native platforms and disabled features:
//!
//! - **Platform Stub**: A fallback stub implementation providing safe, parameter-equivalent
//!   default values when compiled for target platforms where the native implementation is unavailable
//!   (e.g., Web, Mobile, Embedded).
//! - **Feature Stub**: A fallback placeholder implementation designed to compile successfully and
//!   preserve API parity when a specific feature flag (such as `sys-info` or `gui`) is disabled.
//!
//! Categories:
//! - Native Applications (compiled for host OS/hardware) - see native/
//! - Web Applications (browser engine) - future
//! - Mobile Applications (iOS/Android touch paradigms) - future
//! - Embedded Software (dedicated hardware: routers, thermostats, cars, etc.) - future
//!
//! Windows and Linux specifics live here.
//!
//! For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/ARCHITECTURE.md).
//! Cross-platform with native features and platform-specific stubs.



#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PowerStatus {
    pub ac_online: bool,
    pub battery_percent: u8,
}

impl PowerStatus {
    pub const BATTERY_PERCENT_UNKNOWN: u8 = 255;

    pub fn is_battery_percent_unknown(&self) -> bool {
        self.battery_percent == Self::BATTERY_PERCENT_UNKNOWN
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemBiosInfo {
    pub manufacturer: String,
    pub product: String,
    pub model: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiskDriveInfo {
    pub path: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkAdapterInfo {
    pub name: String,
    pub description: String,
    pub ip_addresses: Vec<String>,
    pub adapter_type: String, // "Wi-Fi", "Ethernet", "Bluetooth", "Virtual", "Other"
    pub is_up: bool,
}

/// A standard cross-platform query provider trait for system information and hardware metrics.
///
/// Implemented by web, mobile, embedded, and native modules to enable first-class platform fallbacks
/// and clean compile-time or run-time query dispatching.
pub trait PlatformProvider {
    /// Gets the current screen resolution as (width, height) pixels.
    fn get_system_screen_resolution() -> (i32, i32);

    /// Gets the console window DPI scale factor (default 96).
    fn get_console_window_dpi() -> u32;

    /// Queries the system accent color as (r, g, b) bytes.
    fn query_accent_color() -> (u8, u8, u8);

    /// Checks if high contrast accessibility theme is enabled.
    fn query_high_contrast() -> bool;

    /// Queries the operating system version string.
    fn query_os_version() -> String;

    /// Checks if dark mode is preferred by the operating system / environment.
    fn query_dark_mode() -> bool;

    /// Queries current power source and battery levels if available.
    fn query_power_status() -> Option<PowerStatus>;

    /// Queries BIOS or board identification details if available.
    fn query_bios_info() -> Option<SystemBiosInfo>;

    /// Queries shell environment and terminal emulator names.
    fn query_shell_and_terminal() -> (String, String);

    /// Enumerates local storage disk drives and free space metrics.
    fn query_disk_drives() -> Vec<DiskDriveInfo>;

    /// Gets the names of installed graphics processing units.
    fn query_gpu_names() -> Vec<String>;

    /// Enumerates active and inactive network adapters and IP addresses.
    fn query_network_adapters() -> Vec<NetworkAdapterInfo>;

    /// Lists connected displays and monitor details.
    fn get_all_monitors() -> Vec<String>;
}

#[cfg(all(target_os = "windows", feature = "sys-info"))]
pub use crate::toolkit::sys_info::providers::WindowsPlatform;

#[cfg(all(target_os = "linux", feature = "sys-info"))]
pub use crate::toolkit::sys_info::providers::LinuxPlatform;

#[cfg(all(
    not(any(target_os = "windows", target_os = "linux")),
    not(target_arch = "wasm32"),
    not(any(target_os = "android", target_os = "ios")),
    not(any(target_os = "none", target_os = "uefi")),
    feature = "sys-info"
))]
pub use crate::toolkit::sys_info::providers::FallbackPlatform;

#[cfg(feature = "gpu")]
pub use crate::toolkit::gpu::{init_headless_gpu, run_compute_shader};

pub use crate::toolkit::platform_web::WebPlatform;
pub use crate::toolkit::platform_mobile::MobilePlatform;
pub use crate::toolkit::platform_embedded::EmbeddedPlatform;

#[cfg(all(target_os = "windows", feature = "sys-info"))]
pub type CurrentPlatform = WindowsPlatform;

#[cfg(all(target_os = "linux", feature = "sys-info"))]
pub type CurrentPlatform = LinuxPlatform;

#[cfg(all(not(any(target_os = "windows", target_os = "linux")), target_arch = "wasm32"))]
pub type CurrentPlatform = WebPlatform;

#[cfg(all(not(any(target_os = "windows", target_os = "linux")), any(target_os = "android", target_os = "ios")))]
pub type CurrentPlatform = MobilePlatform;

#[cfg(all(not(any(target_os = "windows", target_os = "linux")), any(target_os = "none", target_os = "uefi")))]
pub type CurrentPlatform = EmbeddedPlatform;

#[cfg(all(
    not(any(target_os = "windows", target_os = "linux")),
    not(target_arch = "wasm32"),
    not(any(target_os = "android", target_os = "ios")),
    not(any(target_os = "none", target_os = "uefi")),
    feature = "sys-info"
))]
pub type CurrentPlatform = FallbackPlatform;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_provider_implementations() {
        // 1. CurrentPlatform
        let res = CurrentPlatform::get_system_screen_resolution();
        assert!(res.0 > 0 && res.1 > 0);
        let dpi = CurrentPlatform::get_console_window_dpi();
        assert!(dpi > 0);
        let _accent = CurrentPlatform::query_accent_color();
        let _hc = CurrentPlatform::query_high_contrast();
        let _os = CurrentPlatform::query_os_version();
        let _dark = CurrentPlatform::query_dark_mode();
        let _power = CurrentPlatform::query_power_status();
        let _bios = CurrentPlatform::query_bios_info();
        let _shell = CurrentPlatform::query_shell_and_terminal();
        let _disks = CurrentPlatform::query_disk_drives();
        let _gpus = CurrentPlatform::query_gpu_names();
        let _network = CurrentPlatform::query_network_adapters();
        let _monitors = CurrentPlatform::get_all_monitors();

        // 2. WebPlatform
        assert_eq!(WebPlatform::get_system_screen_resolution(), (1920, 1080));
        assert_eq!(WebPlatform::get_console_window_dpi(), 96);
        assert_eq!(WebPlatform::query_accent_color(), (0, 120, 215));
        assert_eq!(WebPlatform::query_os_version(), "Web Browser (WASM)");
        assert!(WebPlatform::query_dark_mode());
        assert!(WebPlatform::query_power_status().is_none());

        // 3. MobilePlatform
        assert_eq!(MobilePlatform::get_system_screen_resolution(), (1080, 2400));
        assert_eq!(MobilePlatform::get_console_window_dpi(), 320);
        assert_eq!(MobilePlatform::query_accent_color(), (103, 80, 164));
        assert!(MobilePlatform::query_dark_mode());
        let mobile_power = MobilePlatform::query_power_status().unwrap();
        assert!(!mobile_power.ac_online);
        assert_eq!(mobile_power.battery_percent, 85);

        // 4. EmbeddedPlatform
        assert_eq!(EmbeddedPlatform::get_system_screen_resolution(), (320, 240));
        assert_eq!(EmbeddedPlatform::get_console_window_dpi(), 96);
        assert_eq!(EmbeddedPlatform::query_accent_color(), (0, 255, 0));
        assert_eq!(EmbeddedPlatform::query_os_version(), "Embedded Bare-Metal / RTOS");
        assert!(!EmbeddedPlatform::query_dark_mode());
        let embedded_power = EmbeddedPlatform::query_power_status().unwrap();
        assert!(embedded_power.ac_online);
        assert_eq!(embedded_power.battery_percent, 100);
    }
}
