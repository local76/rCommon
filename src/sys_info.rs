#![allow(dead_code)]

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[derive(Debug, Clone, Default)]
pub struct PowerStatus {
    pub ac_online: bool,
    pub battery_percent: u8,
}

#[derive(Debug, Clone, Default)]
pub struct SystemBiosInfo {
    pub manufacturer: String,
    pub product: String,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct DiskDriveInfo {
    pub path: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphMap {
    pub status_ok: &'static str,
    pub status_err: &'static str,
    pub info: &'static str,
    pub warning: &'static str,
    pub cpu: &'static str,
    pub gpu: &'static str,
    pub memory: &'static str,
    pub disk: &'static str,
    pub package: &'static str,
    pub battery: &'static str,
    pub shell: &'static str,
    pub terminal: &'static str,
    pub network: &'static str,
    pub clipboard: &'static str,
    pub play: &'static str,
    pub play_empty: &'static str,
}

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
    #[cfg(target_os = "windows")]
    {
        windows::get_system_screen_resolution()
    }
    #[cfg(target_os = "linux")]
    {
        linux::get_system_screen_resolution()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        (1920, 1080)
    }
}

pub fn get_console_window_dpi() -> u32 {
    #[cfg(target_os = "windows")]
    {
        windows::get_console_window_dpi()
    }
    #[cfg(not(target_os = "windows"))]
    {
        96
    }
}

pub fn query_os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        windows::query_os_version()
    }
    #[cfg(target_os = "linux")]
    {
        linux::query_os_version()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        "Generic OS".to_string()
    }
}

pub fn query_dark_mode() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows::query_dark_mode()
    }
    #[cfg(target_os = "linux")]
    {
        linux::query_dark_mode()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        true
    }
}

pub fn query_power_status() -> Option<PowerStatus> {
    #[cfg(target_os = "windows")]
    {
        windows::query_power_status()
    }
    #[cfg(target_os = "linux")]
    {
        linux::query_power_status()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

pub fn query_bios_info() -> Option<SystemBiosInfo> {
    #[cfg(target_os = "windows")]
    {
        windows::query_bios_info()
    }
    #[cfg(target_os = "linux")]
    {
        linux::query_bios_info()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

pub fn query_shell_and_terminal() -> (String, String) {
    static SHELL_AND_TERM: std::sync::OnceLock<(String, String)> = std::sync::OnceLock::new();
    SHELL_AND_TERM
        .get_or_init(|| {
            #[cfg(target_os = "windows")]
            {
                windows::query_shell_and_terminal()
            }
            #[cfg(target_os = "linux")]
            {
                linux::query_shell_and_terminal()
            }
            #[cfg(not(any(target_os = "windows", target_os = "linux")))]
            {
                ("sh".to_string(), "xterm".to_string())
            }
        })
        .clone()
}

impl GlyphMap {
    pub fn load() -> Self {
        #[cfg(target_os = "windows")]
        {
            let (_, terminal) = query_shell_and_terminal();
            if terminal == "Windows Console Host" {
                Self {
                    status_ok: "[OK]",
                    status_err: "[ERR]",
                    info: "[i]",
                    warning: "[!]",
                    cpu: "[CPU]",
                    gpu: "[GPU]",
                    memory: "[RAM]",
                    disk: "[DISK]",
                    package: "[PKG]",
                    battery: "[BAT]",
                    shell: "[SH]",
                    terminal: "[TERM]",
                    network: "[NET]",
                    clipboard: "[CLIP]",
                    play: "> ",
                    play_empty: "  ",
                }
            } else {
                Self {
                    status_ok: "✔️",
                    status_err: "❌",
                    info: "ℹ️",
                    warning: "⚠️",
                    cpu: "🧠",
                    gpu: "🎮",
                    memory: "📟",
                    disk: "💾",
                    package: "📦",
                    battery: "🔋",
                    shell: "🐚",
                    terminal: "📟",
                    network: "🌐",
                    clipboard: "📋",
                    play: "▶ ",
                    play_empty: "  ",
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self {
                status_ok: "✔️",
                status_err: "❌",
                info: "ℹ️",
                warning: "⚠️",
                cpu: "🧠",
                gpu: "🎮",
                memory: "📟",
                disk: "💾",
                package: "📦",
                battery: "🔋",
                shell: "🐚",
                terminal: "📟",
                network: "🌐",
                clipboard: "📋",
                play: "▶ ",
                play_empty: "  ",
            }
        }
    }
}

pub fn query_disk_drives() -> Vec<DiskDriveInfo> {
    #[cfg(target_os = "windows")]
    {
        windows::query_disk_drives()
    }
    #[cfg(target_os = "linux")]
    {
        linux::query_disk_drives()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        vec![DiskDriveInfo {
            path: "/".to_string(),
            free_bytes: 50 * 1024 * 1024 * 1024,
            total_bytes: 100 * 1024 * 1024 * 1024,
        }]
    }
}

pub fn query_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}
