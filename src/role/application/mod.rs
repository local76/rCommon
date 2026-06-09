//! Application Software role.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).
//!
//! Task-oriented software for end users or higher level tools.
//! Includes visual effects, games, RGB control for user-facing features.

// Application software role code (visuals, games, RGB for user-facing features).

pub mod rgb;
pub mod game;
pub mod packages;  // Package inventory from helm etc.
pub mod formatting;  // Formatting helpers from helm etc.
pub mod palette;  // Backend-agnostic ScreenPalette for cross-renderer color story
#[cfg(feature = "scenes")]
pub mod scenes;  // The 10 r* screensaver effects (cosmos, glyphs, flame, etc.) — formerly in trance-scenes/. Move your effect source here in 4.1+. Behind `feature = "scenes"`.

// Re-exports
pub use rgb::{RgbColor, OpenRGBDevice, parse_device_payload, OpenRGBConfig, OpenRGBClient, device_type_name, RgbCommand, RgbController};
pub use game::{ObstacleJumpState, ObstacleJumpGame};
pub use packages::{
    count_scoop, count_choco, count_npm, count_steam, count_ms_store, count_native, count_winget, count_dpkg, count_pacman, count_flatpak, count_snap, PackageManager, PACKAGE_MANAGERS, get_packages_breakdown
};
#[cfg(feature = "sys-info")]
pub use formatting::{get_cpu_info, get_memory_info};

pub use formatting::{
    get_host_info, get_formatted_uptime, get_battery_info, get_disks_info, get_gpu_names, detect_shell_and_terminal
};
