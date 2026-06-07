use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

/// A custom progress gauge styled with the system DWM accent color.
#[derive(Debug, Clone)]
pub struct AccentGauge {
    pub progress: f64, // 0.0 to 1.0
    pub label: String,
    pub accent_color: Color,
    pub dim_color: Color,
    pub use_unicode: bool,
}

impl AccentGauge {
    pub fn new(
        progress: f64,
        label: &str,
        accent_color: Color,
        dim_color: Color,
        use_unicode: bool,
    ) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            label: label.to_string(),
            accent_color,
            dim_color,
            use_unicode,
        }
    }
}

impl Widget for AccentGauge {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        if area.height < 1 || area.width < 4 {
            return;
        }

        // Render starting bracket
        buf[(area.x, area.y)]
            .set_char('[')
            .set_fg(self.dim_color);

        // Render ending bracket
        buf[(area.x + area.width - 1, area.y)]
            .set_char(']')
            .set_fg(self.dim_color);

        // Render gauge fill/empty
        let bar_width = area.width - 2;
        let filled_cells = (self.progress * bar_width as f64).round() as u16;
        let fill_symbol = if self.use_unicode { '█' } else { '#' };
        let empty_symbol = if self.use_unicode { '░' } else { '-' };

        for i in 0..bar_width {
            let cx = area.x + 1 + i;
            let cell = &mut buf[(cx, area.y)];
            if i < filled_cells {
                cell.set_char(fill_symbol).set_fg(self.accent_color);
            } else {
                cell.set_char(empty_symbol).set_fg(self.dim_color);
            }
        }

        // Overlay centered text
        let pct_text = format!(" {:.0}% ", self.progress * 100.0);
        let label_text = if !self.label.is_empty() {
            format!(" {} -{}", self.label, pct_text)
        } else {
            pct_text
        };

        let label_len = label_text.chars().count() as u16;
        if label_len < area.width {
            let start_x = (area.width - label_len) / 2;
            for (i, c) in label_text.chars().enumerate() {
                let cx = area.x + start_x + i as u16;
                if cx >= area.x && cx < area.x + area.width {
                    let cell = &mut buf[(cx, area.y)];
                    cell.set_char(c);
                    let bold_style = cell.style().add_modifier(Modifier::BOLD);
                    cell.set_style(bold_style);
                }
            }
        }
    }
}

/// A custom selection list highlighted with the Windows DWM accent color.
#[derive(Debug, Clone)]
pub struct AccentList<'a> {
    pub items: Vec<&'a str>,
    pub selected_index: usize,
    pub accent_color: Color,
    pub dim_color: Color,
    pub active_text_color: Color,
    pub bullet_char: &'a str,
}

impl<'a> AccentList<'a> {
    pub fn new(
        items: Vec<&'a str>,
        selected_index: usize,
        accent_color: Color,
        dim_color: Color,
        active_text_color: Color,
        bullet_char: &'a str,
    ) -> Self {
        Self {
            items,
            selected_index,
            accent_color,
            dim_color,
            active_text_color,
            bullet_char,
        }
    }
}

impl<'a> Widget for AccentList<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::default();
        let inner_area = block.inner(area);

        let mut lines = Vec::new();
        for (idx, item) in self.items.iter().enumerate() {
            if idx >= inner_area.height as usize {
                break;
            }

            if idx == self.selected_index {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {} ", self.bullet_char),
                        Style::default()
                            .fg(self.accent_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        *item,
                        Style::default()
                            .fg(self.active_text_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("   ", Style::default().fg(self.dim_color)),
                    Span::styled(*item, Style::default().fg(self.dim_color)),
                ]));
            }
        }

        Paragraph::new(lines).render(inner_area, buf);
    }
}

/// A custom tab selection header highlighted with the DWM accent color.
#[derive(Debug, Clone)]
pub struct AccentTabs<'a> {
    pub tabs: Vec<&'a str>,
    pub selected_index: usize,
    pub accent_color: Color,
    pub dim_color: Color,
    pub text_color: Color,
}

impl<'a> AccentTabs<'a> {
    pub fn new(
        tabs: Vec<&'a str>,
        selected_index: usize,
        accent_color: Color,
        dim_color: Color,
        text_color: Color,
    ) -> Self {
        Self {
            tabs,
            selected_index,
            accent_color,
            dim_color,
            text_color,
        }
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

            let style = if idx == self.selected_index {
                Style::default()
                    .fg(self.accent_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.dim_color)
            };

            // Render characters
            for (i, c) in label.chars().enumerate() {
                let cx = current_x + i as u16;
                let cell = &mut buf[(cx, area.y)];
                cell.set_char(c).set_style(style);
            }

            // Draw indicator bar on the second row if height allows
            if area.height > 1 && idx == self.selected_index {
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

/// A custom single-line textbox container highlighted with the DWM accent color when focused.
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

/// Category of TUI visual toast notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Warning,
    Error,
    Info,
}

/// An overlay widget to render dynamic visual alerts directly inside a TUI frame layout.
pub struct ToastBox<'a> {
    pub title: &'a str,
    pub message: &'a str,
    pub kind: ToastKind,
    pub accent_color: Color,
    pub dim_color: Color,
    pub text_color: Color,
}

impl<'a> ToastBox<'a> {
    pub fn new(
        title: &'a str,
        message: &'a str,
        kind: ToastKind,
        accent_color: Color,
        dim_color: Color,
        text_color: Color,
    ) -> Self {
        Self {
            title,
            message,
            kind,
            accent_color,
            dim_color,
            text_color,
        }
    }
}

impl<'a> Widget for ToastBox<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        if area.height < 3 || area.width < 10 {
            return;
        }

        // Determine border and status colors based on ToastKind
        let (border_color, icon) = match self.kind {
            ToastKind::Success => (Color::Rgb(0, 255, 127), "✔️"),
            ToastKind::Error => (Color::Rgb(255, 85, 85), "❌"),
            ToastKind::Warning => (Color::Rgb(250, 210, 50), "⚠️"),
            ToastKind::Info => (self.accent_color, "ℹ️"),
        };

        // Render block with borders
        let block_title = format!(" {} {} ", icon, self.title);
        let block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(block_title, Style::default().fg(border_color).add_modifier(Modifier::BOLD)));
        let inner_area = block.inner(area);
        block.render(area, buf);

        // Render message text (wrapped to fit inner_area)
        let words: Vec<&str> = self.message.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut current_line = String::new();
        for word in words {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_line.len() + 1 + word.len() <= inner_area.width as usize {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        for (idx, line) in lines.iter().enumerate() {
            if idx >= inner_area.height as usize {
                break;
            }
            let cx = inner_area.x;
            let cy = inner_area.y + idx as u16;
            
            let truncated: String = line.chars().take(inner_area.width as usize).collect();
            for (char_idx, c) in truncated.chars().enumerate() {
                buf[(cx + char_idx as u16, cy)]
                    .set_char(c)
                    .set_fg(self.text_color);
            }
        }
    }
}

