//! Text wrapping and paragraph alignment utility helpers.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).
//!
//! Helps custom terminal grids, screensavers, and widgets wrap and align paragraphs
//! cleanly without spilling outside of their dedicated rendering boundaries.

/// Supported text alignments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

/// Wraps text into lines that do not exceed `max_width` characters, wrapping at word boundaries.
/// Maintains existing explicit newlines from the input.
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    if max_width == 0 {
        return vec![text.to_string()];
    }
    
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                if word.len() > max_width {
                    // Word is longer than line width, force split it
                    let mut start = 0;
                    while start < word.len() {
                        let end = (start + max_width).min(word.len());
                        lines.push(word[start..end].to_string());
                        start = end;
                    }
                } else {
                    current_line.push_str(word);
                }
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
                if current_line.len() > max_width {
                    // Force split
                    let mut start = 0;
                    while start < current_line.len() {
                        let end = (start + max_width).min(current_line.len());
                        lines.push(current_line[start..end].to_string());
                        start = end;
                    }
                    current_line.clear();
                }
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        } else if paragraph.is_empty() {
            // Keep empty lines from source
            lines.push(String::new());
        }
    }
    lines
}

/// Aligns a single line of text to the specified width using padding.
/// If the text is longer than `width`, it will be truncated.
pub fn align_line(line: &str, width: usize, alignment: TextAlignment) -> String {
    let line_len = line.chars().count();
    if line_len >= width {
        return line.chars().take(width).collect();
    }
    
    let extra_spaces = width - line_len;
    match alignment {
        TextAlignment::Left => {
            format!("{}{}", line, " ".repeat(extra_spaces))
        }
        TextAlignment::Right => {
            format!("{}{}", " ".repeat(extra_spaces), line)
        }
        TextAlignment::Center => {
            let left_pad = extra_spaces / 2;
            let right_pad = extra_spaces - left_pad;
            format!("{}{}{}", " ".repeat(left_pad), line, " ".repeat(right_pad))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text() {
        let text = "Hello world from a very long line of text that should definitely be wrapped.";
        let wrapped = wrap_text(text, 15);
        for line in &wrapped {
            assert!(line.len() <= 15, "Line too long: '{}' (len {})", line, line.len());
        }
        assert_eq!(wrapped[0], "Hello world");
    }

    #[test]
    fn test_align_line() {
        let line = "abc";
        assert_eq!(align_line(line, 5, TextAlignment::Left), "abc  ");
        assert_eq!(align_line(line, 5, TextAlignment::Right), "  abc");
        assert_eq!(align_line(line, 5, TextAlignment::Center), " abc ");
    }
}
