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
//! Also includes retro effects (FallingGlyphs, RisingFlames, FlowingParticles, etc.),
//! TerminalCell grid rendering, and game primitives for terminal UIs.
//! Used by rIdle, rFetch, rMonitor, rTemplate, rWifi, etc.
//!
//! # 4.0 design system
//!
//! In library 4.0 all chrome/theming types live under
//! [`library::interface::tui::design`](design). The legacy `theme` /
//! `widgets` / `status` / `markdown` / `markdown_viewer` / `layout` / `text`
//! paths below are kept as deprecated re-exports for one minor release.
//!
//! For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/ARCHITECTURE.md).
//! Cross-platform with native features and platform-specific stubs.

// -- 4.0 source of truth --
#[cfg(feature = "widgets")]
pub mod widgets;  // The remaining Accent* widgets (gauge, list, scrollbar, tabs, textbox)
#[cfg(feature = "effects")]
pub mod effects;
#[cfg(feature = "effects")]
pub mod screensaver;
pub mod constants;

// 4.0 unified design system. The chrome types (theme, status, toast, layout,
// markdown viewer, layout_guard, title_banner, effect_preview, mouse_selection,
// text, colors) all live under here. See `design/mod.rs` for the full list.
#[cfg(all(feature = "widgets", feature = "effects"))]
pub mod design;
#[cfg(feature = "widgets")]
pub mod design_widgets_only {
    // Some consumers (like pure-widgets CLI helpers) only enable the
    // `widgets` feature without `effects`. The full `design` façade
    // requires both, so we expose a widgets-only subset under a distinct
    // module path to keep the feature graph clean.
    pub use crate::interface::tui::design::*;
}

// -- 3.x back-compat re-exports (deprecated, will be removed in 4.1) --
//
// These mirror the 3.0 paths so existing r* consumers don't break on
// the 4.0 upgrade. New code should use `library::interface::tui::design::*`.
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub mod theme {
    pub use crate::interface::tui::design::theme::*;
}
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub mod markdown {
    pub use crate::interface::tui::design::markdown::*;
}
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub mod markdown_viewer {
    pub use crate::interface::tui::design::markdown_viewer::*;
}
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub mod layout {
    pub use crate::interface::tui::design::layout::*;
}
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub mod status {
    pub use crate::interface::tui::design::status::*;
}
// `text` was unconditional in 3.x. In 4.0 it lives in `design::text` (gated
// on `interface-tui`). For back-compat when only `widgets` is enabled (so
// `design::text` exists), we shim to it. With no features at all, the
// `text` path was never useful (it's ratatui-shaped), so the shim is
// only compiled when the `widgets` feature is on.
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub mod text {
    pub use crate::interface::tui::design::text::*;
}

// -- 3.x re-exports at the tui:: level (deprecated, will be removed in 4.1) --
//
// 3.0 had `use library::interface::tui::ThemeColors;` etc. These are now
// deprecated and route through the design module. Use the design path
// (`library::interface::tui::design::ThemeColors`) in new code.
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub use widgets::{
    AccentColors, AccentTheme, AccentGauge, AccentList, AccentTabs, AccentTextBox, TextBox,
    ToastBox, ToastKind, AccentScrollbar, draw_title_banner, draw_effect_preview, ButtonRect,
    MouseSelection, is_too_small, render_too_small_warning,
};
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub use theme::{ThemeColors, get_theme};
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub use markdown::{parse_markdown_to_lines, draw_markdown_modal};
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub use markdown_viewer::MarkdownViewerState;
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub use status::StatusBar;
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub use layout::{centered_rect, format_help_row};
#[cfg(feature = "effects")]
pub use effects::{
    TuiEffect, Particle, RainDrop, GravityCenter, render_logo_block,
    FallingGlyphs, FlowingParticles, PulledParticles, FallingDroplets, RisingFlames,
    FallingComets, PulsingGlyphs, PulsingWaves, FlowingBlocks, PulledBlocks,
    RisingGlyphs, PulsingParticles,
};
#[cfg(feature = "widgets")]
#[allow(deprecated)]
pub use text::{TextAlignment, wrap_text, align_line, char_width, visible_len, visible_split};
#[cfg(feature = "effects")]
#[allow(deprecated)]
pub use screensaver::{Screensaver, ScreensaverEffect, ScreensaverRenderer, ScreensaverState};
pub use constants::{MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT};
