//! Window drag-to-move helper for borderless console TUIs.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).
//!
//! Encapsulates the 30-line mouse-drag block in every r* TUI: when the user clicks
//! within row 0..=2 of the title bar, capture cursor + window rect as the drag origin;
//! on each Drag event, compute the delta and call `set_window_pos`.

use crate::lifecycle::foreground::window::{get_window_rect, query_cursor_pos, set_window_pos};

/// State for an in-progress drag-to-move gesture.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowDrag {
    pub active: bool,
    /// Cursor position (x, y) at drag start.
    pub start_cursor: Option<(i32, i32)>,
    /// Window top-left (x, y) at drag start.
    pub start_window: Option<(i32, i32)>,
}

impl WindowDrag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if a drag gesture is currently in progress.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Begin a drag if `row` is in the title bar. Returns true on drag-start.
    pub fn try_begin(&mut self, row: u16) -> bool {
        if row > 2 {
            return false;
        }
        if let (Some(cursor), Some(rect)) = (query_cursor_pos(), get_window_rect()) {
            self.active = true;
            self.start_cursor = Some(cursor);
            self.start_window = Some((rect.left, rect.top));
            true
        } else {
            false
        }
    }

    /// Update the window position based on the current cursor delta.
    /// No-op if not currently dragging.
    pub fn update(&mut self) {
        if !self.active {
            return;
        }
        let (Some(start_cursor), Some(start_window)) = (self.start_cursor, self.start_window) else {
            return;
        };
        if let Some(curr_cursor) = query_cursor_pos() {
            let dx = curr_cursor.0 - start_cursor.0;
            let dy = curr_cursor.1 - start_cursor.1;
            set_window_pos(start_window.0 + dx, start_window.1 + dy);
        }
    }

    /// End the current drag gesture.
    pub fn end(&mut self) {
        self.active = false;
        self.start_cursor = None;
        self.start_window = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_inactive() {
        let d = WindowDrag::new();
        assert!(!d.is_active());
    }

    #[test]
    fn test_try_begin_below_title() {
        let mut d = WindowDrag::new();
        // Row > 2 should not start a drag (regardless of native API availability).
        let _ = d.try_begin(10);
        assert!(!d.is_active());
    }
}
