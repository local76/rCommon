//! Effect preview widget for ratatui TUIs.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::core::TerminalCell;

/// Renders a slice of `TerminalCell` grid as a Paragraph on the frame.
pub fn draw_effect_preview(
    f: &mut Frame,
    area: Rect,
    grid: &[TerminalCell],
    eff_cols: usize,
    eff_rows: usize,
) {
    let mut preview_lines: Vec<Line> = Vec::new();
    for r in 0..eff_rows {
        let mut spans = vec![];
        for c in 0..eff_cols {
            let idx = r * eff_cols + c;
            if idx < grid.len() {
                let cell = &grid[idx];
                let mut st = Style::default()
                    .fg(Color::Rgb(cell.fg.0, cell.fg.1, cell.fg.2))
                    .bg(Color::Rgb(cell.bg.0, cell.bg.1, cell.bg.2));
                if cell.bold {
                    st = st.add_modifier(Modifier::BOLD);
                }
                spans.push(Span::styled(cell.ch.to_string(), st));
            }
        }
        preview_lines.push(Line::from(spans));
    }
    f.render_widget(
        Paragraph::new(preview_lines)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Left),
        area,
    );
}
