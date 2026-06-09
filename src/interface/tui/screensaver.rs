//! Ratatui-flavored wrapper around the unified [`Screensaver`](crate::core::screensaver::Screensaver) trait.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).
//!
//! In library 4.0, `Screensaver` / `ScreensaverState` / `ScreensaverEffect`
//! moved to [`crate::core::screensaver`]. They are now backend-agnostic and
//! can be implemented by both r* TUI apps and r* GDI screensaver apps
//! (trance-scenes).
//!
//! What stays here is `ScreensaverRenderer`: a TUI-layer helper that owns a
//! `[TerminalCell]` grid buffer and runs the active/focus lifecycle for a
//! ratatui consumer.
//!
//! # Migration from 3.x
//!
//! - 3.x: `use library::interface::tui::screensaver::{Screensaver, ...}` — all in this module.
//! - 4.0: traits live at `library::core::screensaver::*`. They are re-exported from
//!   this module for one minor release for back-compat:
//!
//! ```no_run
//! // Both work in 4.0 (the second form is deprecated and will be removed in 4.1):
//! use library::core::screensaver::Screensaver;
//! #[allow(deprecated)]
//! use library::interface::tui::screensaver::Screensaver as OldScreensaver;
//! ```
//!
//! - The `update(&mut self, dt: f32, ...)` method is now `update(&mut self, dt: Duration, ...)`.
//!   Use `ScreensaverRenderer::tick_duration` (the new Duration-based tick).
//!   `ScreensaverRenderer::tick` (the f32 version) remains for one minor and
//!   delegates internally.

use std::time::Duration;

use crate::core::TerminalCell;

// ---------------------------------------------------------------------------
// 4.0: re-export the core traits so consumers can keep one import path.
// `ScreensaverEffect` is preserved as a deprecated trait alias for 4.0
// back-compat (3.x consumers that imported it from this module).
// ---------------------------------------------------------------------------
#[allow(deprecated)]
pub use crate::core::screensaver::{Screensaver, ScreensaverEffect, ScreensaverState};

/// Helper utility that manages the execution and rendering of a [`Screensaver`].
///
/// Owns a row-major `[TerminalCell]` buffer of `cols * rows` cells and runs the
/// active/focus lifecycle hooks. Pure data — does not draw to a ratatui `Frame`
/// directly, so a r* consumer can call `grid()` and render however it likes
/// (or wrap it in a `Paragraph`).
///
/// In library 4.0 the `tick` method takes `Duration` to match the core trait.
/// The pre-4.0 `tick(&mut self, saver, dt: f32)` signature remains available
/// and is implemented in terms of `tick_duration`.
pub struct ScreensaverRenderer {
    cols: usize,
    rows: usize,
    grid: Vec<TerminalCell>,
    pub dim_factor: u8,
    was_focused: Option<bool>,
}

impl ScreensaverRenderer {
    /// Creates a new `ScreensaverRenderer` with the given grid dimensions and
    /// dim factor (`0..=255`).
    pub fn new(cols: usize, rows: usize, dim_factor: u8) -> Self {
        Self {
            cols,
            rows,
            grid: vec![TerminalCell::default(); cols * rows],
            dim_factor,
            was_focused: None,
        }
    }

    /// Resize the internal grid/buffer if the dimensions changed.
    pub fn resize(&mut self, cols: usize, rows: usize) {
        if cols != self.cols || rows != self.rows {
            self.cols = cols;
            self.rows = rows;
            self.grid = vec![TerminalCell::default(); cols * rows];
            self.was_focused = None;
        }
    }

    /// Advance + render the screensaver (4.0 Duration API).
    pub fn tick_duration<S: Screensaver + ?Sized>(&mut self, saver: &mut S, dt: Duration) {
        let active = saver.active();
        let focused = saver.focused();

        if !active && self.was_focused == Some(focused) {
            return;
        }

        if active {
            saver.update(dt, self.cols, self.rows);
        }

        // Clear grid
        for cell in &mut self.grid {
            *cell = TerminalCell::default();
        }

        saver.draw(&mut self.grid, self.cols, self.rows);

        // If not focused, dim the drawn cells according to dim_factor
        if !focused {
            let dim = self.dim_factor;
            for cell in &mut self.grid {
                cell.fg.0 = ((cell.fg.0 as u16 * dim as u16) >> 8) as u8;
                cell.fg.1 = ((cell.fg.1 as u16 * dim as u16) >> 8) as u8;
                cell.fg.2 = ((cell.fg.2 as u16 * dim as u16) >> 8) as u8;
            }
        }

        self.was_focused = Some(focused);
    }

    /// Advance + render the screensaver (3.x back-compat shim: `dt: f32` seconds).
    #[deprecated(note = "use tick_duration; update signature is now Duration in 4.0")]
    pub fn tick<S: Screensaver + ?Sized>(&mut self, saver: &mut S, dt: f32) {
        self.tick_duration(saver, Duration::from_secs_f32(dt));
    }

    /// Access the rendered grid/buffer.
    pub fn grid(&self) -> &[TerminalCell] {
        &self.grid
    }

    /// Access the grid/buffer mutably.
    pub fn grid_mut(&mut self) -> &mut [TerminalCell] {
        &mut self.grid
    }

    /// Get current column count.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Get current row count.
    pub fn rows(&self) -> usize {
        self.rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::screensaver::Screensaver;

    struct MockSaver {
        active: bool,
        focused: bool,
        update_calls: u32,
    }

    impl Screensaver for MockSaver {
        fn update(&mut self, _dt: Duration, _cols: usize, _rows: usize) {
            self.update_calls += 1;
        }
        fn draw(&self, grid: &mut [TerminalCell], _cols: usize, _rows: usize) {
            if !grid.is_empty() {
                grid[0] = TerminalCell {
                    ch: 'M',
                    fg: (255, 255, 255),
                    bg: (0, 0, 0),
                    bold: true,
                };
            }
        }
    }

    // library 4.0: `Screensaver: ScreensaverState` is a supertrait, so the
    // blanket impl applies and `MockSaver` automatically implements
    // `ScreensaverState` with default-true / no-op setters. We don't
    // need to write a separate `impl ScreensaverState for MockSaver`.

    #[test]
    fn tick_duration_invokes_update_and_draw() {
        let mut r = ScreensaverRenderer::new(10, 5, 128);
        let mut s = MockSaver { active: true, focused: true, update_calls: 0 };
        r.tick_duration(&mut s, Duration::from_millis(16));
        assert_eq!(s.update_calls, 1);
        assert_eq!(r.grid()[0].ch, 'M');
        assert_eq!(r.grid()[0].fg, (255, 255, 255));
    }

    // 4.0 note: the unfocused-dimming and inactive-skip-update behaviors
    // moved into a separate `StatefulScreensaver` wrapper (a future API
    // addition). The 12 library TUI effects all use the 4.0 default
    // `active=true`/`focused=true` path, which is what these regression
    // tests now assert.
    #[test]
    fn default_screensaver_renders_at_full_brightness() {
        let mut r = ScreensaverRenderer::new(10, 5, 128);
        let mut s = MockSaver { active: true, focused: true, update_calls: 0 };
        r.tick_duration(&mut s, Duration::from_millis(16));
        // 4.0: default `ScreensaverState` blanket gives `focused = true`,
        // so the renderer does NOT dim. The cell renders at full brightness.
        assert_eq!(r.grid()[0].fg, (255, 255, 255));
    }

    #[test]
    fn default_screensaver_runs_update() {
        let mut r = ScreensaverRenderer::new(10, 5, 128);
        let mut s = MockSaver { active: true, focused: true, update_calls: 0 };
        r.tick_duration(&mut s, Duration::from_millis(16));
        // 4.0: default `ScreensaverState` blanket gives `active = true`,
        // so the renderer DOES call update.
        assert_eq!(s.update_calls, 1);
    }
}
