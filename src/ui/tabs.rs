use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use super::colors::AccentColors;

/// A custom tab selection header highlighted with the DWM accent color.
/// Supports `focused: bool` to be first-class in multi-panel/focus UIs (e.g.
/// tab between Diagnostics / Effects panels). When focused the selected tab
/// uses the accent color + bold + underline bar. When inactive the current
/// selection is still visible but rendered in dim (no bar) so the whole
/// header can de-emphasize together with borders.
///
/// # Examples
///
/// ```
/// use library::ui::colors::AccentTheme;
/// use library::ui::tabs::AccentTabs;
///
/// let colors = AccentTheme::default_dark();
/// let tabs = vec!["Dashboard", "System Logs", "Settings"];
/// let is_focused = true;
///
/// let tab_header = AccentTabs::new_with_colors(
///     tabs,
///     0, // Selected tab index
///     &colors,
///     is_focused,
/// );
/// ```
#[derive(Debug, Clone)]
pub struct AccentTabs<'a> {
    pub tabs: Vec<&'a str>,
    pub selected_index: usize,
    pub accent_color: Color,
    pub dim_color: Color,
    pub text_color: Color,
    pub focused: bool,
}

impl<'a> AccentTabs<'a> {
    pub fn new(
        tabs: Vec<&'a str>,
        selected_index: usize,
        accent_color: Color,
        dim_color: Color,
        text_color: Color,
        focused: bool,
    ) -> Self {
        Self {
            tabs,
            selected_index,
            accent_color,
            dim_color,
            text_color,
            focused,
        }
    }

    /// First-class constructor using the bundled `AccentColors`.
    pub fn new_with_colors(
        tabs: Vec<&'a str>,
        selected_index: usize,
        colors: &AccentColors,
        focused: bool,
    ) -> Self {
        Self::new(tabs, selected_index, colors.accent, colors.dim, colors.text, focused)
    }
}

impl<'a> Widget for AccentTabs<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        if area.height < 1 || area.width < 2 {
            return;
        }

        let mut current_x = area.x;
        for (idx, tab) in self.tabs.iter().enumerate() {
            let label = format!("  {}  ", tab);
            let label_len = label.chars().count() as u16;
            if current_x + label_len >= area.x + area.width {
                break;
            }

            let is_selected = idx == self.selected_index;
            let style = if is_selected && self.focused {
                Style::default()
                    .fg(self.accent_color)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                // Current but panel inactive: still visible, but dim (no bold)
                Style::default().fg(self.text_color)
            } else {
                Style::default().fg(self.dim_color)
            };

            // Render characters
            for (i, c) in label.chars().enumerate() {
                let cx = current_x + i as u16;
                let cell = &mut buf[(cx, area.y)];
                cell.set_char(c).set_style(style);
            }

            // Draw indicator bar on the second row if height allows (only when focused)
            if area.height > 1 && is_selected && self.focused {
                for i in 0..label_len {
                    let cx = current_x + i;
                    let cell = &mut buf[(cx, area.y + 1)];
                    cell.set_char('▔').set_fg(self.accent_color); // Unicode overbar character
                }
            }

            current_x += label_len;
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
    fn test_accent_tabs_construction() {
        let colors = AccentColors::new(Color::Blue, Color::Gray, Color::White);
        let tabs = vec!["Tab1", "Tab2"];
        let accent_tabs = AccentTabs::new_with_colors(tabs, 0, &colors, true);
        assert!(accent_tabs.focused);
        assert_eq!(accent_tabs.accent_color, Color::Blue);
    }

    #[test]
    fn test_accent_tabs_focused_vs_unfocused_rendering() {
        let colors = AccentColors::new(Color::Blue, Color::Gray, Color::White);
        let tabs = vec!["Tab1", "Tab2"];

        // Focused tab: should render indicator bar '▔' underneath selected tab
        let tabs_focused = AccentTabs::new_with_colors(tabs.clone(), 0, &colors, true);
        let mut buf_focused = Buffer::empty(Rect::new(0, 0, 15, 2));
        tabs_focused.render(Rect::new(0, 0, 15, 2), &mut buf_focused);
        
        assert_eq!(buf_focused[(2, 1)].symbol(), "▔");
        assert_eq!(buf_focused[(2, 1)].fg, Color::Blue);

        // Unfocused tab: should NOT render indicator bar '▔'
        let tabs_unfocused = AccentTabs::new_with_colors(tabs, 0, &colors, false);
        let mut buf_unfocused = Buffer::empty(Rect::new(0, 0, 15, 2));
        tabs_unfocused.render(Rect::new(0, 0, 15, 2), &mut buf_unfocused);
        
        for x in 0..15 {
            assert_ne!(buf_unfocused[(x, 1)].symbol(), "▔");
        }
    }
}
