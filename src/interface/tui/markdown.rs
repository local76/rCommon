//! Markdown parser and renderer widgets for ratatui TUIs.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use crate::interface::tui::theme::ThemeColors;

/// A lightweight, custom terminal markdown parser returning styled TUI Spans and Lines.
pub fn parse_markdown_to_lines(content: &str, theme: &ThemeColors) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut current_paragraph = String::new();

    // Helper closure to flush the accumulated paragraph text to a single TUI line.
    let flush_paragraph = |para: &mut String, lines: &mut Vec<Line<'static>>| {
        if !para.is_empty() {
            lines.push(Line::from(Span::styled(
                para.clone(),
                Style::default().fg(theme.text_main),
            )));
            para.clear();
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::Rgb(150, 240, 150)),
            )));
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(""));
            continue;
        }

        if let Some(header) = trimmed.strip_prefix("# ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("=== {} ===", header.to_uppercase()),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        } else if let Some(header) = trimmed.strip_prefix("## ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("--- {} ---", header),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        } else if let Some(header) = trimmed.strip_prefix("### ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(Span::styled(
                header.to_string(),
                Style::default().fg(theme.accent),
            )));
        } else if let Some(item) = trimmed
            .strip_prefix("* ")
            .or_else(|| trimmed.strip_prefix("- "))
        {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(vec![
                Span::styled(" • ", Style::default().fg(theme.accent)),
                Span::styled(item.to_string(), Style::default().fg(theme.text_main)),
            ]));
        } else if let Some((num_str, rest)) = trimmed.split_once(". ").filter(|(num_str, _)| !num_str.is_empty() && num_str.chars().all(|c| c.is_ascii_digit())) {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}. ", num_str),
                    Style::default().fg(theme.accent),
                ),
                Span::styled(
                    rest.to_string(),
                    Style::default().fg(theme.text_main),
                ),
            ]));
        } else if let Some(quote) = trimmed.strip_prefix("> ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(Span::styled(
                format!("  │ {}", quote),
                Style::default()
                    .fg(theme.text_dim)
                    .add_modifier(Modifier::ITALIC),
            )));
        } else {
            // Append standard lines to the current paragraph block.
            if !current_paragraph.is_empty() {
                current_paragraph.push(' ');
            }
            current_paragraph.push_str(trimmed);
        }
    }
    flush_paragraph(&mut current_paragraph, &mut lines);
    lines
}

/// Renders a scrollable markdown modal popup with a scrollbar.
/// Returns the updated visible rows count.
pub fn draw_markdown_modal(
    f: &mut Frame,
    filename: &str,
    markdown_lines: &[Line<'static>],
    markdown_scroll: usize,
    theme: &ThemeColors,
    area: Rect,
) -> usize {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use crate::interface::tui::widgets::scrollbar::AccentScrollbar;

    let popup_block = Block::default()
        .title(format!(
            " Document Viewer: {} (Press Esc/q to Close) ",
            filename
        ))
        .title_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    let paragraph = Paragraph::new(markdown_lines.to_vec())
        .block(popup_block)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .alignment(ratatui::layout::Alignment::Left)
        .scroll((markdown_scroll as u16, 0));

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(paragraph, area);

    let inner_height = area.height.saturating_sub(2) as usize;
    let scrollbar = AccentScrollbar::new(
        markdown_scroll,
        markdown_lines.len(),
        inner_height,
        theme.accent,
        theme.border,
    );
    f.render_widget(scrollbar, area);
    inner_height
}

/// Helper macro to compile-time embed a list of markdown documents from a directory.
/// Returns a `HashMap<&'static str, &'static str>` mapping file names to contents.
#[macro_export]
macro_rules! embedded_docs {
    ($folder:expr, [ $($file:expr),* $(,)? ]) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($file, include_str!(concat!($folder, "/", $file)));
        )*
        map
    }};
}
