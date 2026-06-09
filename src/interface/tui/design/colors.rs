use ratatui::style::Color;

/// First-class bundle of accent/dim/text colors for the TUI accent widget family.
/// 
/// This makes it easy for consumers (template panels, pulse dashboards, etc.)
/// to pass a consistent theme to AccentList, AccentTabs, AccentGauge, etc.
/// without repeating 3-4 color args everywhere.
///
/// # Examples
///
/// ```
/// use library::interface::tui::widgets::AccentColors;
/// use ratatui::style::Color;
///
/// // Create from custom colors
/// let colors = AccentColors::new(Color::Cyan, Color::DarkGray, Color::White);
/// assert_eq!(colors.accent, Color::Cyan);
///
/// // Calculate theme colors for dark mode from an accent color
/// let colors = AccentColors::calculate_from_accent(Color::Rgb(0, 245, 255), true);
/// assert_eq!(colors.text, Color::Gray);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccentColors {
    pub accent: Color,
    pub dim: Color,
    pub text: Color,   // main / active text color
}

impl AccentColors {
    pub fn new(accent: Color, dim: Color, text: Color) -> Self {
        Self { accent, dim, text }
    }

    /// Convenience for the common pattern in r* apps (matches get_theme output).
    pub fn from_accent_dim_text(accent: Color, dim: Color, text: Color) -> Self {
        Self::new(accent, dim, text)
    }

    /// Queries the system theme and DWM accent colors dynamically, constructing
    /// a default/dim/text color theme bundle.
    ///
    /// Requires the `sys-info` feature to fetch real OS colors; otherwise falls back
    /// to standard premium ecosystem colors.
    pub fn query_system() -> Self {
        #[cfg(feature = "sys-info")]
        {
            let theme = crate::platform::native::sys_info::query_system_theme();
            let (r, g, b) = theme.accent_color;
            let accent = Color::Rgb(r, g, b);
            
            // Calculate matching dim/text colors depending on light/dark mode
            let (dim, text) = if theme.is_dark_mode {
                let dim_r = (r as f32 * 0.35) as u8;
                let dim_g = (g as f32 * 0.35) as u8;
                let dim_b = (b as f32 * 0.35) as u8;
                (Color::Rgb(dim_r, dim_g, dim_b), Color::Gray)
            } else {
                let dim_r = (r as f32 * 0.7) as u8;
                let dim_g = (g as f32 * 0.7) as u8;
                let dim_b = (b as f32 * 0.7) as u8;
                (Color::Rgb(dim_r, dim_g, dim_b), Color::Black)
            };
            
            Self { accent, dim, text }
        }
        #[cfg(not(feature = "sys-info"))]
        {
            // Fallback premium colors (cyan ecosystem defaults)
            Self {
                accent: Color::Rgb(0, 245, 255), // Sleek Cyan
                dim: Color::Rgb(0, 80, 85),      // Darker Cyan
                text: Color::Gray,
            }
        }
    }

    /// Constructs a standard theme using a custom accent color, automatically calculating
    /// appropriate dim and text colors based on whether dark mode is preferred.
    pub fn calculate_from_accent(accent: Color, is_dark_mode: bool) -> Self {
        let (r, g, b) = match accent {
            Color::Rgb(r, g, b) => (r, g, b),
            Color::Cyan => (0, 245, 255),
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Yellow => (255, 255, 0),
            Color::Magenta => (255, 0, 255),
            _ => (0, 245, 255),
        };
        
        let (dim, text) = if is_dark_mode {
            let dim_r = (r as f32 * 0.35) as u8;
            let dim_g = (g as f32 * 0.35) as u8;
            let dim_b = (b as f32 * 0.35) as u8;
            (Color::Rgb(dim_r, dim_g, dim_b), Color::Gray)
        } else {
            let dim_r = (r as f32 * 0.7) as u8;
            let dim_g = (g as f32 * 0.7) as u8;
            let dim_b = (b as f32 * 0.7) as u8;
            (Color::Rgb(dim_r, dim_g, dim_b), Color::Black)
        };
        
        Self { accent, dim, text }
    }
}

/// Helper facade to query system-wide accent themes and retrieve standard fallbacks,
/// returning an [`AccentColors`] bundle.
///
/// `AccentTheme` acts as a stateless factory, delegating live environment checks to
/// [`AccentColors::query_system`] or providing predefined light/dark fallback configurations.
///
/// > [!NOTE]
/// > This utility queries system settings/registry keys dynamically on each call to [`current`](Self::current).
/// > Registry/DWM queries can be relatively slow. For performance-critical rendering loops,
/// > you should call this once at startup (or on window focus change events) and store
/// > the returned [`AccentColors`] bundle in your own application state.
///
/// # Examples
///
/// ```
/// use library::interface::tui::widgets::AccentTheme;
///
/// // Get the active system theme (expensive query)
/// let colors = AccentTheme::current();
/// println!("Active accent color: {:?}", colors.accent);
///
/// // Or get dark/light fallback defaults directly (cheap static return)
/// let dark_colors = AccentTheme::default_dark();
/// let light_colors = AccentTheme::default_light();
/// ```
pub struct AccentTheme;

impl AccentTheme {
    /// Fetches the current system theme and returns an active `AccentColors` bundle.
    pub fn current() -> AccentColors {
        AccentColors::query_system()
    }

    /// Returns a fallback dark-mode theme using standard apps Cyan.
    pub fn default_dark() -> AccentColors {
        AccentColors {
            accent: Color::Rgb(0, 245, 255),
            dim: Color::Rgb(0, 80, 85),
            text: Color::Gray,
        }
    }

    /// Returns a fallback light-mode theme using standard apps Cyan.
    pub fn default_light() -> AccentColors {
        AccentColors {
            accent: Color::Rgb(0, 180, 200),
            dim: Color::Rgb(180, 230, 240),
            text: Color::Black,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accent_colors_construction() {
        let colors = AccentColors::new(
            Color::Rgb(255, 0, 0),
            Color::Rgb(100, 100, 100),
            Color::Rgb(255, 255, 255),
        );
        assert_eq!(colors.accent, Color::Rgb(255, 0, 0));
        assert_eq!(colors.dim, Color::Rgb(100, 100, 100));
        assert_eq!(colors.text, Color::Rgb(255, 255, 255));
    }

    #[test]
    fn test_accent_theme_and_dynamic_colors() {
        let colors_sys = AccentColors::query_system();
        assert!(colors_sys.accent != Color::Reset);
        assert!(colors_sys.dim != Color::Reset);

        let theme_current = AccentTheme::current();
        assert_eq!(theme_current, colors_sys);

        let dark_theme = AccentTheme::default_dark();
        assert_eq!(dark_theme.accent, Color::Rgb(0, 245, 255));
        
        let light_theme = AccentTheme::default_light();
        assert_eq!(light_theme.accent, Color::Rgb(0, 180, 200));

        let calculated_dark = AccentColors::calculate_from_accent(Color::Cyan, true);
        assert_eq!(calculated_dark.accent, Color::Cyan);
        assert_eq!(calculated_dark.text, Color::Gray);

        let calculated_light = AccentColors::calculate_from_accent(Color::Cyan, false);
        assert_eq!(calculated_light.accent, Color::Cyan);
        assert_eq!(calculated_light.text, Color::Black);
    }
}
