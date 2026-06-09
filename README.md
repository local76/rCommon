# library

> The shared foundation library for the local76 ecosystem.

`library` provides the design system, TUI widgets, Screensaver trait, 10 screensaver scene implementations, the screensaver_runtime (Win32 GDI + Linux raw-termios), RGB controller, system-info helpers, registry abstraction, daemon IPC, and file-logging — all behind a 4-layer taxonomy of Cargo features. Every other local76 crate depends on this one.

`library` is the only Cargo crate in the local76 ecosystem that ships a meaningful public API. The 5 TUI apps and the 10 screensaver shims are consumers; `library` is the producer.

---

## Add as a dependency

For local development (recommended — every consumer in this monorepo uses this form):

```toml
[patch."https://github.com/local76/library.git"]
library = { path = "../library" }

[dependencies]
library = { git = "https://github.com/local76/library.git", branch = "main", features = [...] }
```

The `[patch]` redirects the git URL to the local sibling directory. Edits to library source take effect on the next `cargo build` of the consumer.

For external consumers (CI, release):

```toml
[dependencies]
library = { git = "https://github.com/local76/library.git", tag = "v4.2.1" }
```

The git tag pins to a specific published version. Cargo will not pull a `main` branch that hasn't been tagged.

---

## Cargo features

Features follow the 4-layer taxonomy: `interface-*`, `lifecycle-*`, `platform-*`, `role-*`. Granular features (`widgets`, `effects`, `sys-info`, `reg`, `rgb`, `gpu`) remain for backward compatibility.

### Taxonomy features (preferred)

| Feature | What it enables |
|---|---|
| `interface-tui` | Ratatui rendering, widgets, screensaver scenes |
| `interface-gui` | egui/eframe windowing and styles |
| `interface-api` | Local sockets / IPC helpers |
| `interface-cli` | `clap`, `run_doctor`, scaffold helpers |
| `lifecycle-foreground` | Console window styling, DPI, single-instance guard |
| `lifecycle-background` | Daemon, service, tray, clipboard, event log |
| `platform-native` | sysinfo, winreg, OpenRGB, WiFi, X11, DWM |
| `platform-gpu` | Headless GPU compute, wgpu |
| `role-system` | Hardware / BIOS / service / power diagnostics |
| `role-application` | Palette, RGB, packages, screensaver scenes |
| `screensaver-runtime` | Win32 GDI + raw-termios main loop (host a screensaver process) |
| `scenes` | The 10 scene implementations |
| `winget` | Local winget SQLite scanner |

Default features: `interface-tui`, `interface-api`, `lifecycle-foreground`, `lifecycle-background`, `platform-native`, `role-system`, `role-application`, `scenes`, `winget`.

### Granular features (legacy)

| Feature | What it enables |
|---|---|
| `widgets` | Ratatui widgets (AccentGauge, AccentList, AccentTabs) |
| `effects` | ASCII particle systems (FallingGlyphs, RisingFlames, etc.) |
| `sys-info` | System info helper parsing |
| `reg` | Config storage / registry abstraction |
| `rgb` | OpenRGB protocol + controller handlers |
| `gpu` | Headless GPU compute context |

---

## Usage

### Diagnostic doctor

```rust
use library::interface::cli::run_doctor;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--doctor") {
        run_doctor();
        return;
    }
}
```

### Embed a screensaver scene

Every scene implements the `Screensaver` trait. The 10 scene implementations live at `library::role::application::scenes::<name>`.

```rust
use library::core::screensaver::Screensaver;
use library::core::TerminalCell;
use library::role::application::scenes::glyphs::Glyphs;

let mut effect = Glyphs::new();
let mut grid = vec![TerminalCell::default(); cols * rows];
effect.update(std::time::Duration::from_millis(16), cols, rows);
effect.draw(&mut grid, cols, rows);
```

Available scenes (all in `library::role::application::scenes::*`):

| Module | Type | Default look |
|---|---|---|
| `beams` | `Beams` | 4 colored spotlight cones over a starfield |
| `bounce` | `BhopDashboard` | 3-panel cyberpunk TUI dashboard |
| `flame` | `FireEffect` | Bottom-up cellular-automaton fire |
| `gnats` | `Fireflies` | Boid-style firefly swarm |
| `bursts` | `Fireworks` | City skyline + rockets + bursts |
| `cosmos` | `LifeEffect` | Universe lifecycle simulation |
| `glyphs` | `Glyphs` | Falling katakana+digit rain |
| `disco` | `Party` | Disco ball + rays + VU-meter |
| `storm` | `Pour` | Rain + mountains + lightning |
| `chaos` | `Unstable` | System-logo deconstruction + chaos types |

### Host a screensaver process

```rust
fn main() {
    library::screensaver_runtime::run_main(
        library::role::application::scenes::glyphs::Glyphs::new(),
        "glyphs",
    );
}
```

The `screensaver_runtime` module is gated on the `screensaver-runtime` feature (default-off). Enable it in your `Cargo.toml` if your app needs to host a screensaver process directly. The 10 crates in the [`screensavers`](https://github.com/local76/screensavers) workspace do exactly this.

### Render a TUI effect

All 10 TUI effects follow a **Verb × Noun × Style × Palette** taxonomy:

```rust
use library::interface::tui::effects::{
    FallingGlyphs, Style, Palette, TuiEffect,
};

// Default look (matrix-green with trails)
let mut rain = FallingGlyphs::new(80, 24, 0.35);

// Customized: lens-flare heads, theme-matched color
let mut flare_rain = FallingGlyphs::new(80, 24, 0.35)
    .with_style(Style::Flared)
    .with_palette(Palette::Accent);

flare_rain.update(0.016, 80, 24);
flare_rain.draw(&mut grid, 80, 24);
```

| Dimension | Values | Controls |
|---|---|---|
| Verb | `Falling`, `Rising`, `Flowing`, `Pulled`, `Pulsing` | Motion model |
| Noun | `Glyphs`, `Particles`, `Droplets`, `Comets`, `Blocks`, `Waves` | Visual unit |
| Style | `Solid`, `Trailing`, `Flared` | Render treatment |
| Palette | `Monochrome(r,g,b)`, `Accent`, `Heat` | Color source |
| Speed | `Slow`, `Normal`, `Fast`, `Custom(u8)` | Velocity multiplier (0.25x – 5.0x) |

5 verbs × 6 nouns = 30 base combinations × 9 look variants = 270 total. The matrix is the catalog; most combinations are nonsense, but the catalog is the source of truth.

### Use the design façade

```rust
use library::interface::tui::design::prelude::*;
```

This re-exports the status bar, toast, markdown viewer, theme + accent colors, layout guard, and all 10 canonical TUI effects.

---

## Module path migration (3.x → 4.x)

| Old (3.x) | New (4.x) |
| :--- | :--- |
| `library::widgets::*` | `library::interface::tui::widgets::*` |
| `library::effects::*` | `library::interface::tui::effects::*` |
| `library::window::*` | `library::lifecycle::foreground::window::*` |
| `library::guard::*` | `library::lifecycle::foreground::guard::*` |
| `library::service::*` | `library::lifecycle::background::service::*` |
| `library::sys_info::*` | `library::platform::native::sys_info::*` |
| `library::reg::*` | `library::platform::native::reg::*` |
| `library::rgb::*` | `library::role::application::rgb::*` |
| `library::game::BhopGame` | `library::role::application::game::ObstacleJumpGame` |

---

## Visual standards

All applications in the ecosystem share a cohesive UI style. See [docs/VISUAL_STANDARDS.md](docs/VISUAL_STANDARDS.md) for the icon container layout, neon wireframe elements, and branding asset packaging.

## Embedded markdown docs (F1–F7 in-TUI help)

The `library::embedded_docs!` macro bakes markdown files (README, LICENSE, CONTRIBUTING, etc.) directly into your binary at compile time. The TUI can show help text without reading the filesystem at runtime — which would break in a single-file `.scr` Windows screensaver install. See [docs/EMBEDDED_DOCS.md](docs/EMBEDDED_DOCS.md) for the canonical example and the F1–F7 wiring pattern.

---

## Build

```pwsh
git clone https://github.com/local76/library.git
cd library
cargo build --release
```

For the full local76 build orchestrator, see [`toolkit`](https://github.com/local76/toolkit).

---

## License

MIT. See [LICENSE.md](LICENSE.md).
