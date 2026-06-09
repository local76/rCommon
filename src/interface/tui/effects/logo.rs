//! Centered block-letter logo renderer.
//!
//! **Taxonomy Classification**: Interface (TUI) — original home, but the
//! function is a pure string-transformer with a static cache, so library
//! 4.1 re-exports it from `core::logo_block::render_logo_block` (the
//! canonical home for layer-agnostic pure functions). This module remains
//! as a thin re-export for back-compat with any r* TUI app that imports
//! from the original 4.0 path.

pub use crate::core::logo_block::render_logo_block;

/// Dynamic system information getter.
pub use crate::platform::native::sys_info::get_system_info;
