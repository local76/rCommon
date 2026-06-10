//! Mobile platform stubs.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment - Mobile).
//!
//! **Platform Stub**: This is a fallback stub implementation providing safe, parameter-equivalent default values when compiled for this platform.

use crate::toolkit::platform::{PowerStatus, SystemBiosInfo, DiskDriveInfo, NetworkAdapterInfo, PlatformProvider};

pub fn mobile_stub() {}

pub struct MobilePlatform;

impl PlatformProvider for MobilePlatform {
    fn get_system_screen_resolution() -> (i32, i32) {
        (1080, 2400)
    }

    fn get_console_window_dpi() -> u32 {
        320
    }

    fn query_accent_color() -> (u8, u8, u8) {
        (103, 80, 164) // Material purple accent
    }

    fn query_high_contrast() -> bool {
        false
    }

    fn query_os_version() -> String {
        #[cfg(target_os = "android")]
        {
            "Android OS".to_string()
        }
        #[cfg(target_os = "ios")]
        {
            "iOS".to_string()
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            "Mobile OS (Android/iOS)".to_string()
        }
    }

    fn query_dark_mode() -> bool {
        true
    }

    fn query_power_status() -> Option<PowerStatus> {
        Some(PowerStatus {
            ac_online: false,
            battery_percent: 85,
        })
    }

    fn query_bios_info() -> Option<SystemBiosInfo> {
        None
    }

    fn query_shell_and_terminal() -> (String, String) {
        ("sh".to_string(), "Mobile Terminal".to_string())
    }

    fn query_disk_drives() -> Vec<DiskDriveInfo> {
        vec![DiskDriveInfo {
            path: "/storage/emulated/0".to_string(),
            total_bytes: 128 * 1024 * 1024 * 1024,
            free_bytes: 64 * 1024 * 1024 * 1024,
        }]
    }

    fn query_gpu_names() -> Vec<String> {
        vec!["Mobile GPU (Vulkan/Metal)".to_string()]
    }

    fn query_network_adapters() -> Vec<NetworkAdapterInfo> {
        vec![NetworkAdapterInfo {
            name: "wlan0".to_string(),
            description: "Mobile Wi-Fi Connection".to_string(),
            ip_addresses: vec!["192.168.1.150".to_string()],
            adapter_type: "Wi-Fi".to_string(),
            is_up: true,
        }]
    }

    fn get_all_monitors() -> Vec<String> {
        vec!["Built-in Liquid Retina/AMOLED Screen".to_string()]
    }
}
