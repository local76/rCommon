# library Design System (4.0)

The library 4.0 design system is the single source of truth for the visual
identity of every `r*` app in the local76 apps suite:

- **TUI apps** (helm, pulse, trance, template, scout, ignite, hub) —
  ratatui/buffer-managed, drive effects through `ScreensaverRenderer`.
- **Screensaver apps** (trance-scenes: cosmos, gnats, glyphs, flame, ...) —
  GDI/fullscreen pixel loop, share the same `Screensaver` trait.
- **Future r* apps** (CLI tools, native UIs) — same building blocks, layered
  per the 4-layer taxonomy.

This document describes the design system from an r* app author's point of
view. For the underlying architectural rationale, see
[`ARCHITECTURE.md`](../ARCHITECTURE.md).

---

## Single import path

Every r* app should import its UI from this one path:

```rust
use library::interface::tui::design::prelude::*;
```

This brings in:
- `ThemeColors`, `get_theme`, `accent_color_from_hex`
- `AccentColors`, `AccentTheme`
- `StatusBar`, `ToastBox`, `ToastKind`
- `MarkdownViewerState`, `parse_markdown_to_lines`, `draw_markdown_modal`
- `is_too_small`, `render_too_small_warning`
- `draw_title_banner`, `ButtonRect`, `MouseSelection`
- `draw_effect_preview`, `centered_rect`, `format_help_row`
- `wrap_text`, `align_line`, `char_width`, `visible_len`, `visible_split`,
  `TextAlignment`
- `MIN_TERMINAL_WIDTH`, `MIN_TERMINAL_HEIGHT`
- All 12 canonical TUI effects (`FallingGlyphs`, `RisingFlames`, ...)
- `Screensaver`, `ScreensaverState`, `ScreensaverRenderer`,
  `TuiEffect`, `render_logo_block`

If you only need widgets (no effects), use
`library::interface::tui::design_widgets_only::*` (no `effects` feature
required). For most apps, the full `design::prelude` is the right choice.

---

## Color story

The 4.0 color story is centered on the **system accent** + a derived
`ScreenPalette`. Every r* app pulls the same canonical palette and gets a
visually consistent identity out of the box.

```rust
use library::role::application::palette::{query_current_palette, ScreenPalette};

// Cached, cross-platform, falls back to apps cyan on non-Windows.
let palette: ScreenPalette = query_current_palette();

let bg      = palette.bg;      // (0,0,0) in dark mode
let fg      = palette.fg;      // (248,248,242) in dark mode
let accent  = palette.accent;  // System DWM accent on Windows
let dim     = palette.dim;     // 35% of accent
let hot     = palette.hot;     // accent hue +30°, lightness 0.55
let cool    = palette.cool;    // accent hue -120°, lightness 0.45
let mid     = palette.mid;     // (128,128,128) neutral chrome
let peak    = palette.peak;    // (255,255,255) hot peaks
```

`ScreenPalette` is `role::application`-scoped (backend-agnostic) and uses
plain RGB tuples, so it works in both ratatui `Color::Rgb` and GDI pixel
renderers. The same palette is used by `helm`'s TUI border, `cosmos`'s
GDI particles, and `gnats`'s color story — they're all the same color.

### TUI-typed palette

For TUI effects, `dimensions::Palette` exposes the same color story in a
ratatui-friendly enum:

```rust
use library::interface::tui::effects::dimensions::Palette;

let p = Palette::ACCENT;       // system accent
let p = Palette::ACCENT_DIM;   // 35%-dimmed accent (matches ScreenPalette::dim)
let p = Palette::ACCENT_HOT;   // +30° hue (matches ScreenPalette::hot)
let p = Palette::ACCENT_COOL;  // -120° hue (matches ScreenPalette::cool)
let p = Palette::HEAT;         // cold-to-hot ramp
```

---

## Onboarding: helm

`helm` is the reference consumer. Its `main.rs` and `ui/mod.rs` show the
canonical pattern:

```rust
// crates/helm/src/main.rs
use library::interface::tui::design::prelude::*;
use library::lifecycle::background::file_log;
use library::lifecycle::foreground::panic::set_tui_panic_hook;

fn run_tui(args: CliArgs) -> io::Result<()> {
    file_log::set_log_app_name("helm");
    set_tui_panic_hook();
    // ...
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let mut config = Config::load_or_create();
    // ...
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // ...
    let theme = current_theme(&app);
    // ...
    terminal.draw(|f| ui::draw_ui(f, &mut app))?;
}
```

`current_theme` is a 3-line helper:

```rust
fn current_theme(_app: &App) -> ThemeColors {
    let accent = accent_color_from_hex(&win32::get_win_accent_color());
    get_theme(win32::is_dark_mode(), accent)
}
```

`ui::draw_ui` uses `ThemeColors` to style the rounded border, then
delegates to `specs::generate_specs_lines` and `logos::get_colored_logo_lines`
for the body. The status bar at the bottom is `app.status` (a `StatusBar`)
which auto-resets to its default message after a 4-second decay.

`App` holds a `MarkdownViewerState` (for F1-F7 help docs), a
`MouseSelection` (for drag-to-select + clipboard), and the chrome state
machine. There is **no hand-rolled markdown scroll / show_markdown
triple** anywhere — `MarkdownViewerState` encapsulates it.

---

## Onboarding: trance-scenes

`trance-scenes` screensaver apps share the same `Screensaver` trait as the
TUI effects. The GDI/fullscreen pixel loop is:

```rust
// src/trance-core/src/lib.rs (library 4.0)
pub use library::core::screensaver::Screensaver;
pub use library::core::TerminalCell;
pub use library::role::application::palette::ScreenPalette;

pub fn current_palette() -> ScreenPalette {
    library::role::application::palette::query_current_palette()
}
```

A cosmos-style effect then does:

```rust
use std::time::Duration;
use trance_core::{Screensaver, TerminalCell, current_palette};

pub struct LifeEffect { /* ... */ }

impl Screensaver for LifeEffect {
    fn update(&mut self, dt: Duration, _cols: usize, _rows: usize) {
        let delta = dt.as_secs_f32().min(0.1);
        // ...physics...
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        let palette = current_palette();
        let accent = palette.accent;
        // ...draw into grid using accent / hot / cool / mid colors...
    }
    fn has_scanlines(&self) -> bool { true }  // GDI CRT overlay
}
```

Note the 4.0 signature change: `update` takes `Duration` (was `f32`
seconds in 3.x). The `dt.as_secs_f32()` cast in the body keeps the
floating-point math unchanged.

The btrance-core also exposes `current_palette()` so effects can pull
the same `ScreenPalette` that helm uses. **Future work**: the 10
r* screensaver apps can migrate their hand-rolled HSL color math to
`ScreenPalette::hot` / `ScreenPalette::cool` incrementally.

---

## Onboarding: pulse (template)

`pulse` does not exist yet but the design system supports it out of
the box. A typical TUI dashboard:

```rust
// src/main.rs
use library::interface::tui::design::prelude::*;

fn main() -> io::Result<()> {
    set_tui_panic_hook();
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut app = App::new();
    let theme = get_theme(query_dark_mode(), query_accent_color());
    let mut renderer = ScreensaverRenderer::new(80, 24, 128);
    let mut saver: Box<dyn Screensaver> = Box::new(FallingGlyphs::new(80, 24, 0.5));

    loop {
        terminal.draw(|f| {
            if is_too_small(f.area(), (MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT)) {
                render_too_small_warning(f, f.area(), (f.area().width, f.area().height),
                    (MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT), "pulse", theme.accent);
                return;
            }
            draw_dashboard(f, &mut app, &theme, &mut saver, &mut renderer);
        })?;
        // ...event loop...
        renderer.tick_duration(saver.as_mut(), Duration::from_millis(100));
    }
}
```

This gives you the same 100x35 layout-guard modal, the same
4-second-decay status bar, the same `Screensaver` trait on the
screensaver, the same color story — all from one `use` statement.

---

## 4.0 module map

```
library::interface::tui::design
├── theme           ThemeColors, get_theme, accent_color_from_hex
├── colors          AccentColors, AccentTheme (3-color bundles)
├── status          StatusBar (4-second decay pattern)
├── toast           ToastBox, ToastKind
├── markdown        parse_markdown_to_lines, draw_markdown_modal
├── markdown_viewer MarkdownViewerState (F1-F7 state machine)
├── layout_guard    is_too_small, render_too_small_warning
├── title_banner    draw_title_banner, ButtonRect
├── effect_preview  draw_effect_preview
├── mouse_selection MouseSelection
├── layout          centered_rect, format_help_row
└── text            wrap_text, align_line, char_width, visible_len, ...

library::interface::tui::design::prelude
└── everything above + all 12 effects + Screensaver + ScreensaverRenderer

library::core
├── TerminalCell          (renderer-agnostic character cell)
├── LcgRng                (canonical RNG for effects)
├── SystemInfo / DashboardInfo
├── hsl_to_rgb, rgb_to_hsl
└── screensaver
    ├── Screensaver       (single trait, init/update/draw/has_scanlines)
    ├── ScreensaverState  (active/focused sub-trait)
    └── ScreensaverEffect (deprecated trait alias, back-compat)

library::role::application::palette
└── ScreenPalette         (backend-agnostic RGB-tuple color story)
    ├── from_system(accent, is_dark)  // canonical 4.0
    ├── high_contrast(...)
    ├── default_dark() / default_light()
    └── query_current_palette()        // cross-platform helper
```

---

## 3.x → 4.0 migration cheatsheet

| 3.x path | 4.0 path |
|---|---|
| `library::interface::tui::theme` | `library::interface::tui::design::theme` |
| `library::interface::tui::markdown` | `library::interface::tui::design::markdown` |
| `library::interface::tui::markdown_viewer` | `library::interface::tui::design::markdown_viewer` |
| `library::interface::tui::layout` | `library::interface::tui::design::layout` |
| `library::interface::tui::status` | `library::interface::tui::design::status` |
| `library::interface::tui::text` | `library::interface::tui::design::text` |
| `library::interface::tui::widgets` | `library::interface::tui::widgets` (kept for the Accent* widget family) |
| `library::interface::tui::screensaver` | `library::core::screensaver` |
| `fx.update(0.016, 80, 24)` (f32 seconds) | `fx.update(Duration::from_secs_f32(0.016), 80, 24)` |
| `ScreensaverRenderer::tick(&mut s, 0.1)` (deprecated) | `ScreensaverRenderer::tick_duration(&mut s, Duration::from_secs_f32(0.1))` |
| hand-rolled `(show_markdown, markdown_lines, markdown_scroll)` triple | `MarkdownViewerState` |
| hand-rolled `is_dark_mode()` registry read | `library::platform::native::sys_info::query_dark_mode()` (cross-platform) |
| hand-rolled HSL accent rotation | `ScreenPalette::hot` / `ScreenPalette::cool` |

The 3.x paths are still available as **deprecated** re-exports in
`library::interface::tui::*` and `library::widgets::*` for one minor
release. They will be removed in 4.1.

---

## Testing

The design system ships with `tests/design_facade.rs` in library — 11
tests that exercise the public façade end-to-end (theme, accent, status
bar, toast, layout guard, markdown viewer, text wrap, render logo
block, all 12 effects, etc.). Add similar tests in each r* app to lock
in the contract.

`tests/taxonomy_compliance.rs` (also in library) AST-walks `src/` to
enforce the 4-layer taxonomy — `design/` files cannot import from
`lifecycle/`, `platform/`, or `role/`, so a new design-system addition
that violates the layering will fail the test.
