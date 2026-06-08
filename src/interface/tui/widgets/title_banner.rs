//! Title banner widget for ratatui TUIs.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::interface::tui::theme::ThemeColors;

/// Represents the boundary coordinates of a clickable button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonRect {
    pub y: u16,
    pub x_start: u16,
    pub x_end: u16,
}

impl ButtonRect {
    pub fn new(y: u16, x_start: u16, x_end: u16) -> Self {
        Self { y, x_start, x_end }
    }

    /// Checks if a mouse event coordinate falls inside the button boundary.
    pub fn contains(&self, mouse_row: u16, mouse_col: u16) -> bool {
        mouse_row == self.y && mouse_col >= self.x_start && mouse_col < self.x_end
    }
}

/// Renders a generic title banner with system metadata and interactive buttons.
/// Returns `(help_btn_bounds, quit_btn_bounds)` as `Option<ButtonRect>`.
#[allow(clippy::too_many_arguments)] // Title banner is intentionally a single-pass renderer; all metadata fields are needed to compose the title strip in one call.
pub fn draw_title_banner(
    f: &mut Frame,
    area: Rect,
    theme: &ThemeColors,
    app_title: &str,
    app_name: &str,
    app_version: &str,
    username: &str,
    host_name: &str,
    os_str: &str,
) -> (Option<ButtonRect>, Option<ButtonRect>) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            format!(" {} ", app_title),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let ver_str = format!(" {} v{} ", app_name, app_version);
    let user_host_str = format!("{}@{}", username, host_name);
    let os_str_val = os_str;

    // Calculate dynamic coordinates for " help " and " quit " buttons
    let button_y = area.y + 1;
    let inner_width = area.width.saturating_sub(2) as usize;
    let left_len = ver_str.len() + 3 + user_host_str.len() + 3 + os_str_val.len();
    let right_len = 6 + 3 + 6; // " help " + " │ " + " quit "

    let (title_line, help_btn_bounds, quit_btn_bounds) = if inner_width > left_len + right_len {
        let padding_len = inner_width - (left_len + right_len);
        let padding_str = " ".repeat(padding_len);

        let help_offset = 1 + left_len + padding_len;
        let help_start_x = area.x + help_offset as u16;
        let help_end_x = help_start_x + 6;
        let help = Some(ButtonRect::new(button_y, help_start_x, help_end_x));

        let quit_offset = help_offset + 6 + 3;
        let quit_start_x = area.x + quit_offset as u16;
        let quit_end_x = quit_start_x + 6;
        let quit = Some(ButtonRect::new(button_y, quit_start_x, quit_end_x));

        let line = Line::from(vec![
            Span::styled(
                ver_str,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(
                user_host_str,
                Style::default()
                    .fg(theme.username)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(
                os_str_val.to_string(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(padding_str, Style::default()),
            // Help button: " help " in themed background, black text, underlined 'h'
            Span::styled(
                " ",
                Style::default()
                    .bg(theme.help_btn)
                    .fg(ratatui::style::Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "h",
                Style::default()
                    .bg(theme.help_btn)
                    .fg(ratatui::style::Color::Black)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
            Span::styled(
                "elp ",
                Style::default()
                    .bg(theme.help_btn)
                    .fg(ratatui::style::Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            // Quit button: " quit " in themed background, white text, underlined 'q'
            Span::styled(
                " ",
                Style::default()
                    .bg(theme.quit_btn)
                    .fg(ratatui::style::Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "q",
                Style::default()
                    .bg(theme.quit_btn)
                    .fg(ratatui::style::Color::White)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
            Span::styled(
                "uit ",
                Style::default()
                    .bg(theme.quit_btn)
                    .fg(ratatui::style::Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        (line, help, quit)
    } else {
        let help_offset = 1 + ver_str.len() + 3 + user_host_str.len() + 3 + os_str_val.len() + 3;
        let help_start_x = area.x + help_offset as u16;
        let help_end_x = help_start_x + 6;
        let help = Some(ButtonRect::new(button_y, help_start_x, help_end_x));

        let quit_offset = help_offset + 6 + 3;
        let quit_start_x = area.x + quit_offset as u16;
        let quit_end_x = quit_start_x + 6;
        let quit = Some(ButtonRect::new(button_y, quit_start_x, quit_end_x));

        let line = Line::from(vec![
            Span::styled(
                ver_str,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(
                user_host_str,
                Style::default()
                    .fg(theme.username)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(
                os_str_val.to_string(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            // Help button: " help " in themed background, black text, underlined 'h'
            Span::styled(
                " ",
                Style::default()
                    .bg(theme.help_btn)
                    .fg(ratatui::style::Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "h",
                Style::default()
                    .bg(theme.help_btn)
                    .fg(ratatui::style::Color::Black)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
            Span::styled(
                "elp ",
                Style::default()
                    .bg(theme.help_btn)
                    .fg(ratatui::style::Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            // Quit button: " quit " in themed background, white text, underlined 'q'
            Span::styled(
                " ",
                Style::default()
                    .bg(theme.quit_btn)
                    .fg(ratatui::style::Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "q",
                Style::default()
                    .bg(theme.quit_btn)
                    .fg(ratatui::style::Color::White)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
            Span::styled(
                "uit ",
                Style::default()
                    .bg(theme.quit_btn)
                    .fg(ratatui::style::Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        (line, help, quit)
    };

    f.render_widget(Paragraph::new(title_line).block(title_block), area);
    (help_btn_bounds, quit_btn_bounds)
}
