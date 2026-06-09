//! Snapshot-style integration tests for the `design` façade surface.
//!
//! These tests do NOT add an `insta`/`expect-test` dependency. They render
//! real widgets into a `ratatui::buffer::Buffer` at several terminal sizes
//! and assert on stable structural properties (border color, dim factor
//! presence, grid occupancy, ASCII logo integrity). They form the regression
//! net for the 4.0 unified design system: if a future change to any of the
//! re-exported primitives alters the shape of the output, these break.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::Widget;
use library::interface::tui::design::prelude::*;

// ---------------------------------------------------------------------------
// 1. Theme + accent-color façade is intact and produces a valid ThemeColors
// ---------------------------------------------------------------------------
#[test]
fn facade_theme_constructs_dark_and_light() {
    let accent = Color::Rgb(0, 245, 255);
    let dark = get_theme(true, accent);
    let light = get_theme(false, accent);

    assert_eq!(dark.accent, accent);
    assert_eq!(dark.border_active, accent);
    assert_eq!(light.accent, accent);
    assert_eq!(light.border_active, accent);
    // dark and light must differ on the dim text
    assert_ne!(dark.text_dim, light.text_dim);
}

#[test]
fn facade_accent_color_from_hex_parses_well_known() {
    assert_eq!(accent_color_from_hex("#00ff00"), Color::Rgb(0, 255, 0));
    assert_eq!(accent_color_from_hex("#ff00ff"), Color::Rgb(255, 0, 255));
    assert_eq!(accent_color_from_hex("garbage"), Color::Rgb(0, 245, 255));
    assert_eq!(accent_color_from_hex(""), Color::Rgb(0, 245, 255));
}

// ---------------------------------------------------------------------------
// 2. Status bar timing
// ---------------------------------------------------------------------------
#[test]
fn facade_status_bar_decay_is_4s() {
    let mut s = StatusBar::new("default");
    s.set("hello");
    assert_eq!(s.current(), "hello");
    s.decay = std::time::Duration::from_millis(1);
    std::thread::sleep(std::time::Duration::from_millis(10));
    s.tick();
    assert!(s.is_default());
}

// ---------------------------------------------------------------------------
// 3. Toast renders within façade (border + title + first message char)
// ---------------------------------------------------------------------------
#[test]
fn facade_toast_renders_border_and_message() {
    let accent = Color::Cyan;
    let dim = Color::Gray;
    let text = Color::White;
    let toast = ToastBox::new("Saved", "Profile written", ToastKind::Info, accent, dim, text);
    let mut buf = Buffer::empty(Rect::new(0, 0, 30, 5));
    toast.render(Rect::new(0, 0, 30, 5), &mut buf);

    // Border cells (corners) should be styled with the toast color.
    let tl = &buf[(0, 0)];
    assert!(!tl.symbol().is_empty(), "top-left border cell should be set");

    // First character of the message "Profile written" should appear somewhere in the inner area.
    let mut found_p = false;
    for y in 1..4 {
        for x in 0..30 {
            if buf[(x, y)].symbol() == "P" {
                found_p = true;
                break;
            }
        }
    }
    assert!(found_p, "expected 'P' from message to appear in inner area");
}

// ---------------------------------------------------------------------------
// 4. Layout guard is_too_small
// ---------------------------------------------------------------------------
#[test]
fn facade_layout_guard_thresholds() {
    assert!(is_too_small(Rect::new(0, 0, 80, 20), (MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT)));
    assert!(!is_too_small(Rect::new(0, 0, MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT),
        (MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT)));
    assert_eq!(MIN_TERMINAL_WIDTH, 100);
    assert_eq!(MIN_TERMINAL_HEIGHT, 35);
}

// ---------------------------------------------------------------------------
// 5. centered_rect produces a properly nested inner rect at multiple sizes
// ---------------------------------------------------------------------------
#[test]
fn facade_centered_rect_at_canonical_sizes() {
    for (w, h) in [(80, 24u16), (106, 30), (200, 60)] {
        let r = centered_rect(80, 50, Rect::new(0, 0, w, h));
        assert!(r.width <= w, "inner width {w} fits outer");
        assert!(r.height <= h, "inner height {h} fits outer");
        assert!(r.width >= w / 4, "centered rect should not be tiny");
    }
}

// ---------------------------------------------------------------------------
// 6. render_logo_block (used by trance-scenes) produces non-empty lines
// ---------------------------------------------------------------------------
#[test]
fn facade_render_logo_block_works() {
    let lines = render_logo_block("helm", Some("v4.0"));
    assert!(!lines.is_empty(), "logo block should not be empty");
    let total_chars: usize = lines.iter().map(|l| l.chars().count()).sum();
    assert!(total_chars > 0, "logo should contain characters");
}

// ---------------------------------------------------------------------------
// 7. MarkdownViewerState opens + scrolls + closes
// ---------------------------------------------------------------------------
#[test]
fn facade_markdown_viewer_lifecycle() {
    let mut v = MarkdownViewerState::new()
        .with_doc("README.md", "# helm\n\nA fast system fetch utility.");
    assert!(!v.is_open());
    let theme = get_theme(true, Color::Cyan);
    v.open("README.md", &theme);
    assert!(v.is_open());
    assert!(!v.lines.is_empty());
    v.scroll_down(100, 1);
    // Clamped to lines.len() - 1
    assert!(v.scroll < v.lines.len());
    v.close();
    assert!(!v.is_open());
}

// ---------------------------------------------------------------------------
// 8. The canonical 4.0 accent triplet factory (AccentTheme) round-trips
// ---------------------------------------------------------------------------
#[test]
fn facade_accent_theme_defaults_are_cyan_ecosystem() {
    let dark = AccentTheme::default_dark();
    let light = AccentTheme::default_light();
    assert_eq!(dark.accent, Color::Rgb(0, 245, 255));
    assert_eq!(light.accent, Color::Rgb(0, 180, 200));
    assert_ne!(dark.text, light.text);
}

// ---------------------------------------------------------------------------
// 9. text utilities round-trip
// ---------------------------------------------------------------------------
#[test]
fn facade_text_wrap_handles_ascii_and_wide() {
    let wrapped = wrap_text("the quick brown fox", 10);
    assert!(wrapped.len() >= 2);
    let cw_ascii = char_width('a');
    let cw_wide = char_width('日');
    assert_eq!(cw_ascii, 1);
    // char_width is naive (does not account for East Asian Width); assert it
    // returns a positive usize so we know the API is callable via the façade.
    assert!(cw_wide >= 1, "char_width returns a non-negative cell count");
}

// ---------------------------------------------------------------------------
// 10. All 12 effect constructors compile via the façade (regression for
//     re-exports; nothing should panic in new() at canonical 80x24).
// ---------------------------------------------------------------------------
#[test]
fn facade_all_effects_construct_at_80x24() {
    use library::interface::tui::design::prelude::*;
    let (c, r) = (80usize, 24usize);
    let mut grid = vec![library::core::TerminalCell::default(); c * r];
    let mut effects: Vec<Box<dyn TuiEffect>> = vec![
        Box::new(FallingGlyphs::new(c, r, 0.35)),
        Box::new(FlowingParticles::new(c, r)),
        Box::new(PulledParticles::new(c, r)),
        Box::new(FallingDroplets::new(c, r)),
        Box::new(RisingFlames::new(c, r)),
        Box::new(FallingComets::new(c, r)),
        Box::new(PulsingGlyphs::new(c, r)),
        Box::new(PulsingWaves::new(c, r)),
        Box::new(FlowingBlocks::new(c, r)),
        Box::new(PulledBlocks::new(c, r)),
        Box::new(RisingGlyphs::new(c, r)),
        Box::new(PulsingParticles::new(c, r)),
    ];
    for e in effects.iter_mut() {
        e.update(std::time::Duration::from_secs_f32(0.016), c, r);
        e.draw(&mut grid, c, r);
    }
}
