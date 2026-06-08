//! TUI (Text User Interface) components.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer) + Role (Application Software).
//!
//! # Focus & Active States
//! - **Focused**: Controls visual emphasis. Focused components render with full accent styling (active colors, bold, bright text), while unfocused components dim or de-emphasize their visuals (muted borders, 50% brightness) to preserve visual hierarchy.
//! - **Active**: Controls CPU/resource utilization. Active effects and screensavers update and animate normally. Inactive effects pause execution and yield CPU cycles to conserve system resources.
//! 
//! Provides ratatui-based **first-class accent widgets** (AccentList, AccentTabs,
//! AccentGauge, AccentTextBox, ToastBox) that support focused states for
//! consistent tab/focus/panel UIs.
//!
//! Also includes retro effects (MatrixRain, FireEffect, particles, etc.),
//! TerminalCell grid rendering, and game primitives for terminal UIs.
//! Used by rIdle, rFetch, rMonitor, rTemplate, rWifi, etc.
//!
//! For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/rCommon/ARCHITECTURE.md).
//! Cross-platform with native features and platform-specific stubs.

// The actual implementation files were moved here during cleanup.
// widgets.rs and effects.rs provide the TUI presentation code.

#[cfg(feature = "widgets")]
pub mod widgets;
#[cfg(feature = "widgets")]
pub mod theme;
#[cfg(feature = "widgets")]
pub mod markdown;
#[cfg(feature = "widgets")]
pub mod layout;
#[cfg(feature = "effects")]
pub mod effects;
pub mod text;
#[cfg(feature = "effects")]
pub mod screensaver;
pub mod constants;

// Re-export for convenience under tui::
#[cfg(feature = "widgets")]
pub use widgets::{AccentColors, AccentTheme, AccentGauge, AccentList, AccentTabs, AccentTextBox, TextBox, ToastBox, ToastKind, AccentScrollbar, draw_title_banner, draw_effect_preview};
#[cfg(feature = "widgets")]
pub use theme::{ThemeColors, get_theme};
#[cfg(feature = "widgets")]
pub use markdown::{parse_markdown_to_lines, draw_markdown_modal};
#[cfg(feature = "widgets")]
pub use layout::{centered_rect, format_help_row};
#[cfg(feature = "effects")]
pub use effects::{TuiEffect, Particle, RainDrop, MatrixRain, SimpleParticles, GravityParticles, GravityCenter, RainEffect, FireEffect, render_logo_block};
pub use text::{TextAlignment, wrap_text, align_line};
#[cfg(feature = "effects")]
pub use screensaver::{Screensaver, ScreensaverState, ScreensaverEffect, ScreensaverRenderer};
pub use constants::{MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT};