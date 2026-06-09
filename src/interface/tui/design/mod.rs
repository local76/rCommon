//! library 4.0 unified design system for r* TUI apps.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).
//!
//! In library 4.0 every UI-facing r* app (rFetch, rMonitor, rIdle, rTemplate,
//! rWifi, etc.) imports its chrome from this single module. The 3.0-era
//! scattered paths (`library::interface::tui::theme::*`,
//! `library::interface::tui::StatusBar`, `library::widgets::*`, ...) are
//! preserved as deprecated re-exports at `library::lib.rs` for one minor.
//!
//! # What lives here (4.0)
//!
//! - **Theme**: [`ThemeColors`], [`get_theme`], [`accent_color_from_hex`]
//!   (canonical dark/light palettes for r* TUI chrome).
//! - **Accent bundles**: [`AccentColors`], [`AccentTheme`] (3-color triplet
//!   that the Accent* widget family takes as a single arg).
//! - **Status bar**: [`StatusBar`] (decay-timer pattern shared by every
//!   r* app's "Tab to focus" footer).
//! - **Toast**: [`ToastBox`], [`ToastKind`] (Success / Warning / Error / Info).
//! - **Markdown viewer**: [`MarkdownViewerState`], [`parse_markdown_to_lines`],
//!   [`draw_markdown_modal`] (F1-F7 help overlay).
//! - **Layout guard**: [`is_too_small`], [`render_too_small_warning`]
//!   (100x35 minimum-canvas modal).
//! - **Title banner / buttons**: [`draw_title_banner`], [`ButtonRect`]
//!   (mouse-hit-testable buttons).
//! - **Effect preview**: [`draw_effect_preview`] (renders a `TerminalCell`
//!   grid as a ratatui `Paragraph`).
//! - **Mouse selection**: [`MouseSelection`] (drag-to-select + clipboard).
//! - **Layout helpers**: [`centered_rect`], [`format_help_row`].
//! - **Text utilities**: [`wrap_text`], [`align_line`], [`char_width`],
//!   [`visible_len`], [`visible_split`], [`TextAlignment`].
//! - **Terminal size constants**: [`MIN_TERMINAL_WIDTH`], [`MIN_TERMINAL_HEIGHT`].
//!
//! # Single-import path
//!
//! ```no_run
//! use library::interface::tui::design::prelude::*;
//! use ratatui::style::Color;
//!
//! let theme = get_theme(true, Color::Cyan);
//! let mut status = StatusBar::new("Tab to focus");
//! status.set("hello");
//! let toast = ToastBox::new("Saved", "Profile written", ToastKind::Success,
//!     Color::Cyan, Color::Gray, Color::White);
//! ```
//!
//! # Effects (separate but related)
//!
//! The 12 canonical TUI effects (FallingGlyphs, RisingFlames, etc.) live in
//! [`crate::interface::tui::effects`]. The `design::prelude` also re-exports
//! them so a single `use` brings the whole r* visual identity into scope.

// --- Submodules (4.0 source of truth) ---
pub mod colors;
pub mod effect_preview;
pub mod layout;
pub mod layout_guard;
pub mod markdown;
pub mod markdown_viewer;
pub mod mouse_selection;
pub mod status;
pub mod text;
pub mod theme;
pub mod title_banner;
pub mod toast;

// --- Re-exports (single surface for r* apps) ---
pub use crate::interface::tui::constants::{MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH};
pub use colors::{AccentColors, AccentTheme};
pub use effect_preview::draw_effect_preview;
pub use layout::{centered_rect, format_help_row};
pub use layout_guard::{is_too_small, render_too_small_warning};
pub use markdown::{draw_markdown_modal, parse_markdown_to_lines};
pub use markdown_viewer::MarkdownViewerState;
pub use mouse_selection::MouseSelection;
pub use status::StatusBar;
pub use text::{
    align_line, char_width, visible_len, visible_split, wrap_text, TextAlignment,
};
pub use theme::{accent_color_from_hex, get_theme, ThemeColors};
pub use title_banner::{draw_title_banner, ButtonRect};
pub use toast::{ToastBox, ToastKind};

// --- Effects re-exports (chrome + effects-driven apps in one scope) ---
pub use crate::interface::tui::effects::{
    render_logo_block, FallingComets, FallingDroplets, FallingGlyphs, FlowingBlocks,
    FlowingParticles, GravityCenter, Particle, PulledBlocks, PulledParticles, PulsingGlyphs,
    PulsingParticles, PulsingWaves, RainDrop, RisingFlames, RisingGlyphs, TuiEffect,
};

/// Convenience glob-import for r* app `use library::interface::tui::design::prelude::*;`.
pub mod prelude {
    pub use super::{
        accent_color_from_hex, align_line, char_width, centered_rect, draw_effect_preview,
        draw_markdown_modal, draw_title_banner, format_help_row, get_theme, is_too_small,
        parse_markdown_to_lines, render_logo_block, render_too_small_warning, visible_len,
        visible_split, wrap_text, AccentColors, AccentTheme, ButtonRect, FallingComets,
        FallingDroplets, FallingGlyphs, FlowingBlocks, FlowingParticles, GravityCenter,
        MarkdownViewerState, MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH, MouseSelection, Particle,
        PulledBlocks, PulledParticles, PulsingGlyphs, PulsingParticles, PulsingWaves, RainDrop,
        RisingFlames, RisingGlyphs, StatusBar, TextAlignment, ThemeColors, ToastBox, ToastKind,
        TuiEffect,
    };
}
