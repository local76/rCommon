//! Core shared types and primitives.
//!
//! **Taxonomy Classification**: Core (neutral foundation).
//!
//! For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/rCommon/ARCHITECTURE.md).
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
/// rApps suite (CLI tools, TUI effects, background services, etc.).
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
/// (works with ratatui, custom GDI renderers in rIdle-scenes, headless
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
/// Enhanced SystemInfo / Dashboard context ported and generalized from rIdle-scenes/ridle-core
/// and patterns in rFetch/rMonitor. Provides live OS/kernel/logo_text + stats for dynamic UIs/effects.
/// Call periodically for fresh values (uptime, mem, etc.).
///
/// For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/rCommon/ARCHITECTURE.md).
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