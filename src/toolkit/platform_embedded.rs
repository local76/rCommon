//! Embedded platform stubs.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment - Embedded).
//!
//! **Platform Stub**: This is a fallback stub implementation providing safe, parameter-equivalent default values when compiled for this platform.

use crate::toolkit::platform::{PowerStatus, SystemBiosInfo, DiskDriveInfo, NetworkAdapterInfo, PlatformProvider};

pub fn embedded_stub() {}

pub struct EmbeddedPlatform;

impl PlatformProvider for EmbeddedPlatform {
    fn get_system_screen_resolution() -> (i32, i32) {
        (320, 240)
    }

    fn get_console_window_dpi() -> u32 {
        96
    }

    fn query_accent_color() -> (u8, u8, u8) {
        (0, 255, 0) // Monochromatic retro LCD green
    }

    fn query_high_contrast() -> bool {
        false
    }

    fn query_os_version() -> String {
        "Embedded Bare-Metal / RTOS".to_string()
    }

    fn query_dark_mode() -> bool {
        false
    }

    fn query_power_status() -> Option<PowerStatus> {
        Some(PowerStatus {
            ac_online: true,
            battery_percent: 100,
        })
    }

    fn query_bios_info() -> Option<SystemBiosInfo> {
        Some(SystemBiosInfo {
            manufacturer: "Embedded MCU Board".to_string(),
            product: "STM32/ESP32 System".to_string(),
            model: "V1.0.0".to_string(),
        })
    }

    fn query_shell_and_terminal() -> (String, String) {
        ("none".to_string(), "UART Serial Console".to_string())
    }

    fn query_disk_drives() -> Vec<DiskDriveInfo> {
        vec![DiskDriveInfo {
            path: "flash://".to_string(),
            total_bytes: 16 * 1024 * 1024,
            free_bytes: 4 * 1024 * 1024,
        }]
    }

    fn query_gpu_names() -> Vec<String> {
        vec!["SPI/I2C Framebuffer".to_string()]
    }

    fn query_network_adapters() -> Vec<NetworkAdapterInfo> {
        vec![NetworkAdapterInfo {
            name: "eth0".to_string(),
            description: "Embedded MAC/PHY".to_string(),
            ip_addresses: vec!["192.168.0.10".to_string()],
            adapter_type: "Ethernet".to_string(),
            is_up: true,
        }]
    }

    fn get_all_monitors() -> Vec<String> {
        vec!["SPI TFT Screen".to_string()]
    }
}
