//! Wireless Network interfaces, types, and stubs.
//!
//! **Taxonomy Classification**: Platform (WLAN / Hardware Interface).

#[allow(unused_imports)]
use std::fmt::{Debug, Formatter};

#[cfg(any(not(windows), not(feature = "windows-sys")))]
pub mod windows_sys {
    pub mod core {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub struct GUID {
            pub data1: u32,
            pub data2: u16,
            pub data3: u16,
            pub data4: [u8; 8],
        }
    }
}

/// Representation of a scanned wireless network.
#[derive(Clone)]
pub struct WlanNetwork {
    pub ssid: String,
    pub signal_quality: u32,
    pub is_connected: bool,
    pub has_profile: bool,
    pub security_enabled: bool,
    pub auth_algorithm: String,
    pub cipher_algorithm: String,
    pub interface_guid: windows_sys::core::GUID,
}

impl std::fmt::Debug for WlanNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WlanNetwork")
            .field("ssid", &self.ssid)
            .field("signal_quality", &self.signal_quality)
            .field("is_connected", &self.is_connected)
            .field("has_profile", &self.has_profile)
            .field("security_enabled", &self.security_enabled)
            .field("auth_algorithm", &self.auth_algorithm)
            .field("cipher_algorithm", &self.cipher_algorithm)
            .field("interface_guid", &format!(
                "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                self.interface_guid.data1,
                self.interface_guid.data2,
                self.interface_guid.data3,
                self.interface_guid.data4[0],
                self.interface_guid.data4[1],
                self.interface_guid.data4[2],
                self.interface_guid.data4[3],
                self.interface_guid.data4[4],
                self.interface_guid.data4[5],
                self.interface_guid.data4[6],
                self.interface_guid.data4[7],
            ))
            .finish()
    }
}

impl PartialEq for WlanNetwork {
    fn eq(&self, other: &Self) -> bool {
        self.ssid == other.ssid
            && self.signal_quality == other.signal_quality
            && self.is_connected == other.is_connected
            && self.has_profile == other.has_profile
            && self.security_enabled == other.security_enabled
            && self.auth_algorithm == other.auth_algorithm
            && self.cipher_algorithm == other.cipher_algorithm
            && self.interface_guid.data1 == other.interface_guid.data1
            && self.interface_guid.data2 == other.interface_guid.data2
            && self.interface_guid.data3 == other.interface_guid.data3
            && self.interface_guid.data4 == other.interface_guid.data4
    }
}

impl Eq for WlanNetwork {}

/// Hardware and software state of the wireless radio.
#[allow(dead_code)]
pub struct RadioState {
    pub software_on: bool,
    pub hardware_on: bool,
}

/// Helper to truncate a string to a max length and append "..." if exceeded.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let mut truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        truncated.push_str("...");
        truncated
    } else {
        s.to_string()
    }
}

#[cfg(all(windows, feature = "windows-sys"))]
pub mod win32;

#[cfg(not(windows))]
pub mod stub;

#[cfg(all(windows, feature = "windows-sys"))]
pub use win32::*;

#[cfg(not(windows))]
pub use stub::*;
