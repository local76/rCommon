#![allow(dead_code)]

pub struct BorderlessConsole;
impl BorderlessConsole {
    pub fn enable() -> Self { BorderlessConsole }
}

pub struct ConsoleTitleGuard;
impl ConsoleTitleGuard {
    pub fn new(_title: &str) -> Self { ConsoleTitleGuard }
}

pub struct SingleInstanceGuard;
impl SingleInstanceGuard {
    pub fn try_new() -> Result<Self, &'static str> { Ok(SingleInstanceGuard) }
}

pub fn center_console_window() {}
pub fn relaunch_in_conhost_if_needed() {}
pub fn query_dark_mode() -> bool { true }
pub fn get_dwm_accent_color() -> ratatui::style::Color { ratatui::style::Color::Cyan }
pub fn query_cursor_pos() -> Option<(i32, i32)> { None }
pub fn get_window_rect() -> Option<()> { None }
pub fn set_window_pos(_x: i32, _y: i32) {}
pub fn show_toast_notification(_title: &str, _message: &str) {}
pub fn log_windows_event(_source: &str, _event_type: u16, _event_id: u32, _msg: &str) {}
pub fn copy_text_to_clipboard(_text: &str) -> std::io::Result<()> { Ok(()) }
pub fn query_os_version() -> String { "Linux".to_string() }

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

impl GlyphMap {
    pub fn load() -> Self {
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

pub struct PowerStatus {
    pub ac_online: bool,
}

pub fn query_power_status() -> Option<PowerStatus> {
    Some(PowerStatus { ac_online: true })
}

pub fn query_bios_info() -> Option<String> { Some("Linux Kernel".to_string()) }
pub fn query_shell_and_terminal() -> (String, String) { ("sh".to_string(), "xterm".to_string()) }
pub fn get_system_screen_resolution() -> (i32, i32) { (1920, 1080) }
pub fn get_console_window_dpi() -> u32 { 96 }
pub fn query_disk_drives() -> Vec<String> { vec!["/".to_string()] }
pub fn query_local_ip() -> Option<String> { Some("127.0.0.1".to_string()) }
pub fn query_windows_service_status(_service: &str) -> String { "Running".to_string() }
