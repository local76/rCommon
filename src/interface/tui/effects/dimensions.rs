//! Visual dimensions for TUI effects.
//!
//! **Taxonomy Classification**: Interface (TUI) configuration. Used by all
//! effects in `interface::tui::effects` to standardize their render treatment
//! and color source.
//!
//! Each effect exposes a `style: Style` and `palette: Palette` field, with
//! sensible defaults. The same `Style` and `Palette` are used by all effects
//! so consumers can mix-and-match without learning per-effect enums.
//!
//! # Dimensions
//!
//! - [`Style`]: how a single particle/element renders (flat, with trail, with flare).
//! - [`Palette`]: where the color comes from (fixed, system accent, heat-mapped).

use std::sync::OnceLock;

static ACCENT_CACHE: OnceLock<(u8, u8, u8)> = OnceLock::new();

/// Returns the OS accent color (Windows DWM accent or a sensible default on
/// other platforms). Cached for the process lifetime.
pub fn accent_color() -> (u8, u8, u8) {
    if let Some(cached) = ACCENT_CACHE.get() {
        return *cached;
    }
    let resolved = query_accent_color_platform();
    let _ = ACCENT_CACHE.set(resolved);
    resolved
}

#[cfg(feature = "sys-info")]
fn query_accent_color_platform() -> (u8, u8, u8) {
    // query_accent_color returns (r, g, b). On non-Windows or on error, the
    // platform helper returns (0, 120, 215) as a sensible blue default.
    crate::platform::native::sys_info::query_accent_color()
}

#[cfg(not(feature = "sys-info"))]
fn query_accent_color_platform() -> (u8, u8, u8) {
    (0, 120, 215)
}

/// Render treatment for individual cells/elements of a TUI effect.
///
/// All values are mutually exclusive — an effect uses exactly one style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Style {
    /// Flat single character, single color. No decoration.
    #[default]
    Solid,
    /// Each moving element leaves a fading tail behind it.
    /// Drives the look of matrix rain, comets, motion-blurred effects.
    Trailing,
    /// Lens-flare style: a bright leading point with radial spokes.
    /// Use for cinematic / sci-fi / magic highlights.
    Flared,
}

/// Color source for a TUI effect.
///
/// The color returned by [`resolve_color`] depends on the variant and a
/// `t` parameter (typically 0.0..=1.0, e.g. position along a trail, age,
/// temperature). All variants interpret `t` the same way:
/// - `Monochrome`: ignored, returns the fixed color.
/// - `Accent`: ignored, returns the OS accent color.
/// - `AccentDim`: ignored, returns the 35%-dimmed OS accent color (matches
///   the `dim` channel of `library::role::application::palette::ScreenPalette`).
/// - `AccentHot`: ignored, returns the +30° hue-rotated accent at lightness 0.55
///   (matches the `hot` channel of `ScreenPalette`).
/// - `AccentCool`: ignored, returns the -120° hue-rotated accent at lightness 0.45
///   (matches the `cool` channel of `ScreenPalette`).
/// - `Heat`: `t = 0.0` is cold (deep blue), `t = 1.0` is hot (white-hot).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Palette {
    /// Single fixed color, RGB.
    Monochrome(u8, u8, u8),
    /// Pull color from the OS accent color (DWM/registry on Windows).
    Accent,
    /// 35%-dimmed OS accent. Mirrors `ScreenPalette::dim`.
    AccentDim,
    /// +30° hue-rotated OS accent at lightness 0.55. Mirrors `ScreenPalette::hot`.
    AccentHot,
    /// -120° hue-rotated OS accent at lightness 0.45. Mirrors `ScreenPalette::cool`.
    AccentCool,
    /// Cold-to-hot color map (blue -> cyan -> yellow -> red -> white).
    /// `t` in 0.0..=1.0.
    Heat,
}

impl Default for Palette {
    fn default() -> Self {
        Palette::Monochrome(255, 255, 255)
    }
}

impl Palette {
    /// Monochrome white. Equivalent to `Palette::default()`.
    pub const WHITE: Palette = Palette::Monochrome(255, 255, 255);

    /// Monochrome green (classic matrix color).
    pub const GREEN: Palette = Palette::Monochrome(0, 255, 0);

    /// Monochrome blue.
    pub const BLUE: Palette = Palette::Monochrome(80, 160, 255);

    /// Monochrome red.
    pub const RED: Palette = Palette::Monochrome(255, 60, 60);

    /// Heat palette (cold-to-hot).
    pub const HEAT: Palette = Palette::Heat;

    /// OS accent palette.
    pub const ACCENT: Palette = Palette::Accent;

    /// OS-accent dim (35% of accent). Mirrors `ScreenPalette::dim`.
    pub const ACCENT_DIM: Palette = Palette::AccentDim;

    /// OS-accent hot (+30° hue, lightness 0.55). Mirrors `ScreenPalette::hot`.
    pub const ACCENT_HOT: Palette = Palette::AccentHot;

    /// OS-accent cool (-120° hue, lightness 0.45). Mirrors `ScreenPalette::cool`.
    pub const ACCENT_COOL: Palette = Palette::AccentCool;
}

/// Speed preset for an effect. All effects accept this as a `speed: Speed`
/// field; `update()` multiplies the internal velocity by the preset's
/// multiplier so consumers can slow down snow vs. speed up rain without
/// recompiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Speed {
    /// 0.25x — slow, ambient, e.g. lazy snow.
    Slow,
    /// 1.0x — default for the effect.
    #[default]
    Normal,
    /// 2.0x — energetic, e.g. fast rain.
    Fast,
    /// Custom multiplier (clamped to a sane range at use).
    Custom(u8),
}

impl Speed {
    /// Convert to a float multiplier in roughly [0.1, 5.0].
    pub fn multiplier(self) -> f32 {
        match self {
            Speed::Slow => 0.25,
            Speed::Normal => 1.0,
            Speed::Fast => 2.0,
            Speed::Custom(v) => (v as f32 / 100.0).clamp(0.1, 5.0),
        }
    }
}

/// Motion direction for an effect. Used by verbs that have a clear "flow"
/// (Falling, Rising, Flowing). Verbs that are inherently direction-agnostic
/// (Pulled, Pulsing) ignore this field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    /// Top to bottom (default for Falling).
    #[default]
    Down,
    /// Bottom to top (default for Rising).
    Up,
    /// Left to right.
    Right,
    /// Right to left.
    Left,
    /// Top-left to bottom-right (only for `Flowing`-style effects).
    DiagonalDown,
    /// Bottom-left to top-right.
    DiagonalUp,
}

impl Direction {
    /// Returns a (vx_sign, vy_sign) tuple. 0 means "no motion on that axis."
    /// Effects multiply this by their internal speed.
    pub fn signs(self) -> (f32, f32) {
        match self {
            Direction::Down => (0.0, 1.0),
            Direction::Up => (0.0, -1.0),
            Direction::Right => (1.0, 0.0),
            Direction::Left => (-1.0, 0.0),
            Direction::DiagonalDown => (0.707, 0.707),
            Direction::DiagonalUp => (0.707, -0.707),
        }
    }
}

/// Density preset for an effect. Controls how many particles/elements the
/// effect spawns. Mirrors [`Speed`] but affects count rather than velocity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Density {
    /// 0.5x — sparse, e.g. light snow.
    Sparse,
    /// 1.0x — default.
    #[default]
    Normal,
    /// 2.0x — heavy, e.g. storm.
    Dense,
    /// Custom multiplier (v/100, clamped 0.1-5.0).
    Custom(u8),
}

impl Density {
    /// Multiplier applied to the default particle/block count.
    pub fn multiplier(self) -> f32 {
        match self {
            Density::Sparse => 0.5,
            Density::Normal => 1.0,
            Density::Dense => 2.0,
            Density::Custom(v) => (v as f32 / 100.0).clamp(0.1, 5.0),
        }
    }
}

/// Resolve a [`Palette`] to an RGB triple at intensity `t` (0.0..=1.0).
///
/// - `Monochrome(r, g, b)` returns `(r, g, b)` regardless of `t`.
/// - `Accent` returns the cached OS accent color regardless of `t`.
/// - `AccentDim` returns the 35%-dimmed OS accent regardless of `t`.
/// - `AccentHot` returns the +30° hue-rotated accent at lightness 0.55.
/// - `AccentCool` returns the -120° hue-rotated accent at lightness 0.45.
/// - `Heat` interpolates the cold-to-hot ramp based on `t` (clamped).
pub fn resolve_color(palette: Palette, t: f32) -> (u8, u8, u8) {
    match palette {
        Palette::Monochrome(r, g, b) => (r, g, b),
        Palette::Accent => accent_color(),
        Palette::AccentDim => {
            let (r, g, b) = accent_color();
            (
                (r as f32 * 0.35) as u8,
                (g as f32 * 0.35) as u8,
                (b as f32 * 0.35) as u8,
            )
        }
        Palette::AccentHot => {
            let (r, g, b) = accent_color();
            let (h, _s, _l) = crate::core::rgb_to_hsl(r, g, b);
            crate::core::hsl_to_rgb((h + 30.0).rem_euclid(360.0), 0.95, 0.55)
        }
        Palette::AccentCool => {
            let (r, g, b) = accent_color();
            let (h, _s, _l) = crate::core::rgb_to_hsl(r, g, b);
            crate::core::hsl_to_rgb((h - 120.0).rem_euclid(360.0), 0.95, 0.45)
        }
        Palette::Heat => heat_color(t),
    }
}

/// Compute a heat-map color: cold (deep blue) at t=0, hot (white) at t=1.
///
/// Stops:
/// - 0.00 -> (  0,   0, 128)  deep blue
/// - 0.25 -> (  0, 200, 255)  cyan
/// - 0.50 -> (  0, 255,   0)  green
/// - 0.75 -> (255, 200,   0)  yellow-orange
/// - 1.00 -> (255, 255, 255)  white-hot
pub fn heat_color(t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let stops: [(f32, (u8, u8, u8)); 5] = [
        (0.00, (0, 0, 128)),
        (0.25, (0, 200, 255)),
        (0.50, (0, 255, 0)),
        (0.75, (255, 200, 0)),
        (1.00, (255, 255, 255)),
    ];
    for window in stops.windows(2) {
        let (t0, c0) = window[0];
        let (t1, c1) = window[1];
        if t <= t1 {
            let span = (t1 - t0).max(f32::EPSILON);
            let local = (t - t0) / span;
            return (
                lerp_u8(c0.0, c1.0, local),
                lerp_u8(c0.1, c1.1, local),
                lerp_u8(c0.2, c1.2, local),
            );
        }
    }
    stops.last().unwrap().1
}

#[inline]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_default_is_white() {
        assert_eq!(Palette::default(), Palette::Monochrome(255, 255, 255));
    }

    #[test]
    fn test_style_default_is_solid() {
        assert_eq!(Style::default(), Style::Solid);
    }

    #[test]
    fn test_resolve_monochrome() {
        assert_eq!(resolve_color(Palette::Monochrome(10, 20, 30), 0.5), (10, 20, 30));
    }

    #[test]
    fn test_heat_endpoints() {
        let cold = heat_color(0.0);
        let hot = heat_color(1.0);
        // Cold has more blue than red
        assert!(cold.2 > cold.0);
        // Hot is white
        assert_eq!(hot, (255, 255, 255));
    }

    #[test]
    fn test_heat_clamps() {
        let below = heat_color(-1.0);
        let above = heat_color(2.0);
        assert_eq!(below, heat_color(0.0));
        assert_eq!(above, heat_color(1.0));
    }

    #[test]
    fn test_speed_default_and_multipliers() {
        assert_eq!(Speed::default(), Speed::Normal);
        assert_eq!(Speed::Slow.multiplier(), 0.25);
        assert_eq!(Speed::Normal.multiplier(), 1.0);
        assert_eq!(Speed::Fast.multiplier(), 2.0);
        assert!(Speed::Custom(50).multiplier() < 1.0);
        assert!(Speed::Custom(255).multiplier() <= 5.0);
    }

    #[test]
    fn test_direction_default_and_signs() {
        assert_eq!(Direction::default(), Direction::Down);
        assert_eq!(Direction::Down.signs(), (0.0, 1.0));
        assert_eq!(Direction::Up.signs(), (0.0, -1.0));
        assert_eq!(Direction::Right.signs(), (1.0, 0.0));
        assert_eq!(Direction::Left.signs(), (-1.0, 0.0));
        // Diagonals are unit-length on each axis
        let (dx, dy) = Direction::DiagonalDown.signs();
        assert!((dx * dx + dy * dy - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_density_default_and_multipliers() {
        assert_eq!(Density::default(), Density::Normal);
        assert_eq!(Density::Sparse.multiplier(), 0.5);
        assert_eq!(Density::Normal.multiplier(), 1.0);
        assert_eq!(Density::Dense.multiplier(), 2.0);
    }
}
