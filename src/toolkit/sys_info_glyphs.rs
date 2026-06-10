//! Glyph mapping helpers for shell/terminal representations.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment - Native) + Role (System Software).


#[cfg(target_os = "windows")]
use super::query_shell_and_terminal;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_map_load() {
        let glyphs = GlyphMap::load();
        assert!(!glyphs.status_ok.is_empty());
        assert!(!glyphs.status_err.is_empty());
        assert!(!glyphs.info.is_empty());
        assert!(!glyphs.warning.is_empty());
        assert!(!glyphs.cpu.is_empty());
        assert!(!glyphs.gpu.is_empty());
        assert!(!glyphs.memory.is_empty());
        assert!(!glyphs.disk.is_empty());
        assert!(!glyphs.package.is_empty());
        assert!(!glyphs.battery.is_empty());
        assert!(!glyphs.shell.is_empty());
        assert!(!glyphs.terminal.is_empty());
        assert!(!glyphs.network.is_empty());
        assert!(!glyphs.clipboard.is_empty());
        assert!(!glyphs.play.is_empty());
        assert!(!glyphs.play_empty.is_empty());
    }
}

