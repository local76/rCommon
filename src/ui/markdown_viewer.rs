//! Markdown viewer state machine for ratatui TUIs.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).
//!
//! Encapsulates the 3 fields (show_markdown, markdown_lines, markdown_scroll) and the
//! F1-F7 documentation bindings that are duplicated across every r* TUI.

use super::markdown::{draw_markdown_modal, parse_markdown_to_lines};
use super::theme::ThemeColors;
use ratatui::{Frame, layout::Rect, text::Line};
use std::collections::HashMap;

/// State machine + keybindings for the F1-F7 in-TUI markdown documentation viewer.
///
/// # Examples
/// ```
/// use library::ui::markdown_viewer::MarkdownViewerState;
/// use library::ui::theme::get_theme;
/// use ratatui::style::Color;
/// let mut v = MarkdownViewerState::new();
/// v.docs.insert("README.md", "# hello");
/// let theme = get_theme(true, Color::Cyan);
/// v.open("README.md", &theme);
/// assert!(v.is_open());
/// ```
#[derive(Debug, Default, Clone)]
pub struct MarkdownViewerState {
    /// Currently displayed doc filename (None = closed).
    pub filename: Option<String>,
    /// Parsed `Line`s of the open doc.
    pub lines: Vec<Line<'static>>,
    /// Scroll offset.
    pub scroll: usize,
    /// Compile-time-embedded docs (typically populated by `embedded_docs!`).
    pub docs: HashMap<&'static str, &'static str>,
}

impl MarkdownViewerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a doc so it can be opened by filename.
    pub fn with_doc(mut self, name: &'static str, content: &'static str) -> Self {
        self.docs.insert(name, content);
        self
    }

    /// Returns true if the viewer is currently showing a document.
    pub fn is_open(&self) -> bool {
        self.filename.is_some()
    }

    /// Open `name` if it was registered. Resets scroll to top.
    pub fn open(&mut self, name: &'static str, theme: &ThemeColors) {
        if let Some(content) = self.docs.get(name).copied() {
            self.lines = parse_markdown_to_lines(content, theme);
            self.filename = Some(name.to_string());
            self.scroll = 0;
        }
    }

    /// Close the viewer.
    pub fn close(&mut self) {
        self.filename = None;
        self.lines.clear();
        self.scroll = 0;
    }

    /// Scroll up by `n` lines.
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    /// Scroll down by `n` lines, clamped to `lines.len() - visible_rows`.
    pub fn scroll_down(&mut self, n: usize, visible_rows: usize) {
        let max_scroll = self.lines.len().saturating_sub(visible_rows);
        if self.scroll < max_scroll {
            self.scroll = (self.scroll + n).min(max_scroll);
        }
    }

    /// Handle a function-key press (F1..F7 -> corresponding doc name). The caller is
    /// responsible for mapping other keys (Esc closes, arrows scroll).
    pub fn handle_f_key<F: FnOnce(&'static str) -> Option<&'static str>>(
        &mut self,
        f_num: u8,
        theme: &ThemeColors,
        f_to_name: F,
    ) -> bool {
        if let Some(name) = f_to_name(match f_num {
            1 => "F1",
            2 => "F2",
            3 => "F3",
            4 => "F4",
            5 => "F5",
            6 => "F6",
            7 => "F7",
            _ => return false,
        }) {
            self.open(name, theme);
            return true;
        }
        false
    }

    /// Render the markdown modal. Returns the inner height used (for scroll math).
    pub fn render(&self, f: &mut Frame, area: Rect, theme: &ThemeColors) -> usize {
        let Some(name) = self.filename.as_deref() else {
            return 0;
        };
        draw_markdown_modal(f, name, &self.lines, self.scroll, theme, area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::tui::theme::get_theme;
    use ratatui::style::Color;

    #[test]
    fn test_open_close() {
        let mut v = MarkdownViewerState::new().with_doc("README.md", "# title\n\nbody");
        let theme = get_theme(true, Color::Cyan);
        v.open("README.md", &theme);
        assert!(v.is_open());
        assert!(!v.lines.is_empty());
        v.close();
        assert!(!v.is_open());
    }

    #[test]
    fn test_scroll() {
        let mut v = MarkdownViewerState::new().with_doc("R", "# t\n\nbody line");
        let theme = get_theme(true, Color::Cyan);
        v.open("R", &theme);
        v.scroll_down(5, 5);
        assert_eq!(v.scroll, 0); // clamped
    }
}
