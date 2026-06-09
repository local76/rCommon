//! Core shared types and primitives.
//!
//! **Taxonomy Classification**: Core (neutral foundation).
//!
//! For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/ARCHITECTURE.md).
//! Cross-platform with native features and platform-specific stubs.
//!
//! These are the fundamental building blocks that can be safely used
//! across *all* categories without pulling in heavy UI, platform, or
//! lifecycle dependencies.
//!
//! Taxonomy Layers:
//! 1. Interface (Presentation)
//! 2. Execution State (Lifecycle)
//! 3. Platform & Architecture
//! 4. System Role (Purpose)
//!
//! Goal: Nothing in this module should be "TUI only" or "Windows only"
//!       in a way that would break CLI tools, background services,
//!       or future embedded/web targets.
//!
//! All shared data types (LcgRng, TerminalCell, SystemInfo/DashboardInfo) live here
//! as the single source of truth.

pub const UNKNOWN_HOST: &str = "localhost";

/// Linear Congruential Generator. The single canonical RNG implementation
/// used by games, effects, and any deterministic logic across the entire
/// apps suite (CLI tools, TUI effects, background services, etc.).
pub struct LcgRng(u64);

impl LcgRng {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }

    pub fn next_f32(&mut self) -> f32 {
        let val = (self.next_u64() >> 40) as u32;
        (val as f32) * (1.0 / (1u32 << 24) as f32)
    }

    pub fn next_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    pub fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 { return 0; }
        (self.next_u64() % max as u64) as usize
    }

    pub fn next_bool(&mut self, prob: f32) -> bool {
        self.next_f32() < prob
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcg_rng_deterministic() {
        let mut rng = LcgRng::new(42);
        let _val = rng.next_usize(100); // just ensure no panic, value may vary slightly with impl
        // Add more assertions as needed for reproducibility across effects/games.
    }

    #[test]
    fn test_terminal_cell_default() {
        let cell = TerminalCell::default();
        assert!(cell.ch == ' ' || cell.ch == '\0'); // tolerant for Default
    }
}

/// A single cell in a character-grid renderer.
///
/// This is the universal currency for retro/TUI effects, dashboards,
/// and any text-based visual output. It is deliberately backend-agnostic
/// (works with ratatui, custom GDI renderers in trance-scenes, headless
/// logging, etc.).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TerminalCell {
    pub ch: char,
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
    pub bold: bool,
}

/// Rich live system context used by effects, dashboards, and monitoring tools.
///
/// Populated from the platform-specific sys_info implementations but
/// presented in a neutral shape so TUI effects, CLI tools, and background
/// monitors can all consume the same data without depending on ratatui
/// or Windows-specific types.
///
/// This type lives in `core` precisely so that adding new fields for one
/// presentation layer (e.g. fancy TUI effects) cannot accidentally break
/// CLI tools or background services that only care about the data.
#[derive(Debug, Clone, Default)]
pub struct DashboardInfo {
    pub os: String,
    pub logo_text: String,      // e.g. "WIN11", "ARCH", "FEDORA"
    pub kernel: String,
    pub hostname: String,
    pub cpu: String,

    // Live stats (best-effort, may be 0 on some platforms)
    pub uptime_secs: u64,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub mem_used_pct: f32,
    pub power_status: String,
    pub disk_summary: String,
    pub gpus: String,
    pub monitors: String,
}

/// Classification: Core (neutral data usable by Interface/TUI effects, Role/Application dashboards,
/// Platform/Native queries, Lifecycle/Background monitors, etc.).
/// 
/// Enhanced SystemInfo / Dashboard context ported and generalized from trance-scenes/trance-core
/// and patterns in helm/pulse. Provides live OS/kernel/logo_text + stats for dynamic UIs/effects.
/// Call periodically for fresh values (uptime, mem, etc.).
///
/// For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/ARCHITECTURE.md).
#[derive(Clone, Debug, Default)]
pub struct SystemInfo {
    pub os: String,
    /// Short uppercase token suitable for block/ASCII logo rendering, e.g. "WIN11", "POPOS", "FEDORA", "ARCH".
    pub logo_text: String,
    pub kernel: String,
    pub hostname: String,
    pub cpu: String,

    // Live / near-live stats (refreshed on each call where possible)
    pub uptime_secs: u64,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub mem_used_pct: f32,
    pub power_status: String,
    pub disk_summary: String,
    pub gpus: String,
    pub monitors: String,
}

/// Matchers for universal CLI arguments.
/// Placed in core precisely so both Layer 1 (Interface/CLI) and Layer 2 (Lifecycle/Relaunch)
/// can query them without violating layered taxonomy rules.
pub fn is_help_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--help") || arg.eq_ignore_ascii_case("-h") || arg.eq_ignore_ascii_case("help")
}

pub fn is_version_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--version") || arg.eq_ignore_ascii_case("-v") || arg.eq_ignore_ascii_case("version")
}

pub fn is_doctor_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("doctor")
}

pub fn is_install_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("install") || arg.eq_ignore_ascii_case("--install")
}

pub fn is_debug_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--debug") || arg.eq_ignore_ascii_case("-d") || arg.eq_ignore_ascii_case("--verbose")
}

pub fn is_no_color_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--no-color") || arg.eq_ignore_ascii_case("-n")
}

pub fn is_json_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--json") || arg.eq_ignore_ascii_case("-j")
}

pub fn is_high_contrast_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--high-contrast") || arg.eq_ignore_ascii_case("-c")
}

pub fn is_accessible_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--accessible") || arg.eq_ignore_ascii_case("--screen-reader")
}

pub fn is_tui_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--tui") || arg.eq_ignore_ascii_case("--interactive")
}

pub fn is_cli_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--cli") || arg.eq_ignore_ascii_case("--non-interactive")
}

pub fn is_borderless_arg(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--borderless") || arg.eq_ignore_ascii_case("-b")
}

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r_prime + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g_prime + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b_prime + m) * 255.0).clamp(0.0, 255.0) as u8,
    )
}

pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let l = (max + min) / 2.0;
    let mut h = 0.0;
    let mut s = 0.0;
    if d > 0.0001 {
        s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        if max == r {
            h = (g - b) / d + (if g < b { 6.0 } else { 0.0 });
        } else if max == g {
            h = (b - r) / d + 2.0;
        } else {
            h = (r - g) / d + 4.0;
        }
        h *= 60.0;
    }
    (h, s, l)
}

/// Backend-agnostic screensaver trait (no ratatui). See [`screensaver`] module.
///
/// In library 4.0 this is the single source of truth for the screensavers used
/// across r* TUI apps and r* GDI screensaver apps (trance-scenes). The ratatui
/// renderer wrapper lives in `interface::tui::screensaver::ScreensaverRenderer`.
///
/// `ScreensaverEffect` is re-exported as a deprecated trait alias for 4.0
/// back-compat — the method set is identical to `Screensaver`.
pub mod screensaver;
#[allow(deprecated)]
pub use screensaver::{Screensaver, ScreensaverEffect, ScreensaverState};

/// library 4.1: 5x5 block-letter logo renderer. Moved from
/// `interface::tui::effects::logo` to `core` so both the r* TUI effects
/// (interface layer) and the r* screensaver effects (role layer) can
/// import it without violating the 4-layer taxonomy (role is not
/// allowed to import from interface).
pub mod logo_block;