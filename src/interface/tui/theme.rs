//! Theme coloring utility and factory for ratatui-based TUIs.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).

use ratatui::style::Color;

/// Theme color definitions for styling TUI panels and text.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub border: Color,
    pub border_active: Color,
    pub text_main: Color,
    pub text_dim: Color,
    pub accent: Color,
    pub username: Color,
    pub help_btn: Color,
    pub quit_btn: Color,
    pub warning: Color,
    pub success: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}

/// Factory function to retrieve light or dark theme presets.
pub fn get_theme(dark: bool, accent_color: Color) -> ThemeColors {
    if dark {
        ThemeColors {
            border: Color::Rgb(68, 68, 84),
            border_active: accent_color,
            text_main: Color::Rgb(248, 248, 242),
            text_dim: Color::Rgb(136, 136, 153),
            accent: accent_color,
            username: Color::Rgb(255, 215, 0),
            help_btn: Color::Rgb(250, 210, 50),
            quit_btn: Color::Rgb(255, 85, 85), // Red
            warning: Color::Rgb(255, 165, 0),  // Amber/Orange
            success: Color::Rgb(0, 255, 127),
            selection_bg: Color::Rgb(0, 120, 215),
            selection_fg: Color::White,
        }
    } else {
        ThemeColors {
            border: Color::Rgb(180, 180, 190),
            border_active: accent_color,
            text_main: Color::Rgb(40, 42, 54),
            text_dim: Color::Rgb(100, 100, 115),
            accent: accent_color,
            username: Color::Rgb(218, 165, 32),
            help_btn: Color::Rgb(204, 153, 0),
            quit_btn: Color::Rgb(200, 50, 50), // Red
            warning: Color::Rgb(220, 100, 0),  // Amber/Orange
            success: Color::Rgb(0, 180, 90),
            selection_bg: Color::Rgb(180, 215, 255),
            selection_fg: Color::Rgb(40, 42, 54),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors_coverage() {
        let accent = Color::Cyan;
        
        // 1. Test Dark Theme
        let dark = get_theme(true, accent);
        assert_eq!(dark.accent, accent);
        assert_eq!(dark.success, Color::Rgb(0, 255, 127));
        assert_eq!(dark.selection_bg, Color::Rgb(0, 120, 215));
        assert_eq!(dark.warning, Color::Rgb(255, 165, 0));
        assert_eq!(dark.quit_btn, Color::Rgb(255, 85, 85));
        // Verify quit_btn and warning are different semantic colors
        assert_ne!(dark.warning, dark.quit_btn);

        // 2. Test Light Theme
        let light = get_theme(false, accent);
        assert_eq!(light.accent, accent);
        assert_eq!(light.success, Color::Rgb(0, 180, 90));
        assert_eq!(light.selection_bg, Color::Rgb(180, 215, 255));
        assert_eq!(light.warning, Color::Rgb(220, 100, 0));
        assert_eq!(light.quit_btn, Color::Rgb(200, 50, 50));
        // Verify quit_btn and warning are different semantic colors
        assert_ne!(light.warning, light.quit_btn);
    }
}
