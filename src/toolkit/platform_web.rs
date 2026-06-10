//! Web (browser/WASM) platform stubs.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment - Web).
//!
//! **Platform Stub**: This is a fallback stub implementation providing safe, parameter-equivalent default values when compiled for this platform.

use crate::toolkit::platform::{PowerStatus, SystemBiosInfo, DiskDriveInfo, NetworkAdapterInfo, PlatformProvider};

pub fn web_stub() {
    // Placeholder for web-specific sys info, etc.
}

pub struct WebPlatform;

impl PlatformProvider for WebPlatform {
    fn get_system_screen_resolution() -> (i32, i32) {
        (1920, 1080)
    }

    fn get_console_window_dpi() -> u32 {
        96
    }

    fn query_accent_color() -> (u8, u8, u8) {
        (0, 120, 215)
    }

    fn query_high_contrast() -> bool {
        false
    }

    fn query_os_version() -> String {
        "Web Browser (WASM)".to_string()
    }

    fn query_dark_mode() -> bool {
        true
    }

    fn query_power_status() -> Option<PowerStatus> {
        None
    }

    fn query_bios_info() -> Option<SystemBiosInfo> {
        None
    }

    fn query_shell_and_terminal() -> (String, String) {
        ("WebConsole".to_string(), "Browser Tab".to_string())
    }

    fn query_disk_drives() -> Vec<DiskDriveInfo> {
        vec![DiskDriveInfo {
            path: "LocalStorage".to_string(),
            total_bytes: 5 * 1024 * 1024,
            free_bytes: 5 * 1024 * 1024,
        }]
    }

    fn query_gpu_names() -> Vec<String> {
        vec!["WebGL Context GPU".to_string()]
    }

    fn query_network_adapters() -> Vec<NetworkAdapterInfo> {
        vec![NetworkAdapterInfo {
            name: "Browser Network Connection".to_string(),
            description: "WebSocket / HTTP Fetch".to_string(),
            ip_addresses: vec!["127.0.0.1".to_string()],
            adapter_type: "Other".to_string(),
            is_up: true,
        }]
    }

    fn get_all_monitors() -> Vec<String> {
        vec!["Primary Browser Viewport".to_string()]
    }
}
