use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Widget},
};
use super::colors::AccentColors;

/// A custom single-line textbox container highlighted with the DWM accent color when focused.
///
/// # Examples
///
/// ```
/// use rcommon::interface::tui::widgets::{AccentTheme, AccentTextBox};
///
/// let colors = AccentTheme::default_dark();
/// let is_focused = true;
///
/// let textbox = AccentTextBox::new_with_colors(
///     "Search query",
///     "Type here to search...",
///     &colors,
///     is_focused,
/// );
/// ```
#[derive(Debug, Clone)]
pub struct AccentTextBox<'a> {
    pub text: &'a str,
    pub placeholder: &'a str,
    pub focused: bool,
    pub accent_color: Color,
    pub dim_color: Color,
    pub text_color: Color,
}

impl<'a> AccentTextBox<'a> {
    pub fn new(
        text: &'a str,
        placeholder: &'a str,
        focused: bool,
        accent_color: Color,
        dim_color: Color,
        text_color: Color,
    ) -> Self {
        Self {
            text,
            placeholder,
            focused,
            accent_color,
            dim_color,
            text_color,
        }
    }

    /// First-class constructor using the bundled `AccentColors`.
    pub fn new_with_colors(
        text: &'a str,
        placeholder: &'a str,
        colors: &AccentColors,
        focused: bool,
    ) -> Self {
        Self::new(text, placeholder, focused, colors.accent, colors.dim, colors.text)
    }
}

impl<'a> Widget for AccentTextBox<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        if area.height < 3 || area.width < 4 {
            return;
        }

        // Determine border style
        let border_color = if self.focused { self.accent_color } else { self.dim_color };
        let border_style = Style::default().fg(border_color);

        // Render block with borders
        let block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(border_style);
        let inner_area = block.inner(area);
        block.render(area, buf);

        // Render contents
        let is_empty = self.text.is_empty();
        let display_text = if is_empty { self.placeholder } else { self.text };
        let text_style = if is_empty {
            Style::default().fg(self.dim_color).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(self.text_color)
        };

        // Truncate text if it is longer than the inner area width
        let mut char_vec: Vec<char> = display_text.chars().collect();
        let max_width = inner_area.width as usize;
        
        // Handle cursor rendering
        let show_cursor = self.focused;
        if show_cursor && !is_empty && char_vec.len() < max_width {
            char_vec.push('▕'); // block cursor symbol
        } else if show_cursor && is_empty {
            char_vec.insert(0, '▕');
        }

        let slice_len = std::cmp::min(char_vec.len(), max_width);
        for i in 0..slice_len {
            let cx = inner_area.x + i as u16;
            let cell = &mut buf[(cx, inner_area.y)];
            cell.set_char(char_vec[i]);
            
            // Highlight cursor character
            if char_vec[i] == '▕' {
                cell.set_style(Style::default().fg(self.accent_color).add_modifier(Modifier::BOLD));
            } else {
                cell.set_style(text_style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    #[test]
    fn test_accent_textbox_construction() {
        let colors = AccentColors::new(Color::Blue, Color::Gray, Color::White);
        let textbox = AccentTextBox::new_with_colors("hello", "placeholder", &colors, true);
        assert!(textbox.focused);
        assert_eq!(textbox.accent_color, Color::Blue);

        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 5));
        textbox.render(Rect::new(0, 2, 30, 3), &mut buf);
    }

    #[test]
    fn test_accent_textbox_cursor_rendering() {
        let colors = AccentColors::new(Color::Red, Color::Gray, Color::White);

        // Focused empty textbox: should render cursor '▕' at the beginning
        let textbox_focused_empty = AccentTextBox::new_with_colors("", "placeholder", &colors, true);
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 3));
        textbox_focused_empty.render(Rect::new(0, 0, 15, 3), &mut buf);
        assert_eq!(buf[(1, 1)].symbol(), "▕");
        assert_eq!(buf[(1, 1)].fg, Color::Red);

        // Unfocused empty textbox: should NOT render cursor, should render placeholder instead
        let textbox_unfocused_empty = AccentTextBox::new_with_colors("", "placeholder", &colors, false);
        let mut buf_unfocused = Buffer::empty(Rect::new(0, 0, 15, 3));
        textbox_unfocused_empty.render(Rect::new(0, 0, 15, 3), &mut buf_unfocused);
        assert_ne!(buf_unfocused[(1, 1)].symbol(), "▕");
        assert_eq!(buf_unfocused[(1, 1)].symbol(), "p"); // 'p' from "placeholder"

        // Focused non-empty textbox: should render text followed by cursor '▕'
        let textbox_focused_text = AccentTextBox::new_with_colors("hello", "placeholder", &colors, true);
        let mut buf_text = Buffer::empty(Rect::new(0, 0, 15, 3));
        textbox_focused_text.render(Rect::new(0, 0, 15, 3), &mut buf_text);
        assert_eq!(buf_text[(1, 1)].symbol(), "h");
        assert_eq!(buf_text[(6, 1)].symbol(), "▕");
        assert_eq!(buf_text[(6, 1)].fg, Color::Red);
    }
}

/// Standard text entry box state and event handler for Ratatui TUIs.
#[derive(Debug, Clone, Default)]
pub struct TextBox {
    /// Current string content buffer.
    pub text: String,
    /// Index of the typing cursor within the string buffer.
    pub cursor_pos: usize,
    /// Whether this textbox is active and capturing character keystrokes.
    pub active: bool,
}

impl TextBox {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process keystrokes to edit the buffer and move the cursor.
    pub fn handle_key(&mut self, code: crossterm::event::KeyCode) {
        if !self.active {
            return;
        }
        match code {
            crossterm::event::KeyCode::Char(c) => {
                self.text.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            crossterm::event::KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.text.remove(self.cursor_pos);
                }
            }
            crossterm::event::KeyCode::Delete => {
                if self.cursor_pos < self.text.len() {
                    self.text.remove(self.cursor_pos);
                }
            }
            crossterm::event::KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            crossterm::event::KeyCode::Right => {
                if self.cursor_pos < self.text.len() {
                    self.cursor_pos += 1;
                }
            }
            crossterm::event::KeyCode::Home => {
                self.cursor_pos = 0;
            }
            crossterm::event::KeyCode::End => {
                self.cursor_pos = self.text.len();
            }
            _ => {}
        }
    }

    /// Clear the text content buffer and reset cursor to index zero.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
    }
}
