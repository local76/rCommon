# rCommon

`rCommon` is the shared foundation library for the local76 `rApps` ecosystem of local-first terminal and system utilities (including `rIdle`, `rFetch`, `rMonitor`, `rTemplate`, `rWifi`, and `rStartup`).

---

## 🛠️ Installation

Add `rCommon` as a git dependency in your `Cargo.toml`:

```toml
[dependencies]
rcommon = { git = "https://github.com/local76/rCommon.git", branch = "main" }
```

Or for local development, patch it:

```toml
[patch.crates-io]
rcommon = { path = "../rCommon" }
```

---

## 📦 Cargo Features

To prevent dependency and binary size bloat in simple CLI daemons, features are split into taxonomy-aligned gates:

### Taxonomy Features (Preferred)
* `interface-tui` - Enables TUI rendering, widgets, and matrix/particle effects.
* `interface-gui` - Enables graphical UI windowing and styles (egui/eframe).
* `interface-api` - Enables local sockets/IPC services helpers.
* `lifecycle-foreground` - Enables standalone console window styling, DPI adjustments, and single-instance guards.
* `lifecycle-background` - Enables background daemons, system service mutations, and tray/clipboard monitoring.
* `platform-native` - Enables Windows Registry emulation, hardware displays query, and system info parsing.
* `platform-gpu` - Enables headless GPU compute shaders and wgpu interfaces.
* `role-system` - Low-level hardware, bios, service, and power diagnostics.
* `role-application` - High-level utilities, package inventory counting, and OpenRGB integrations.

### Granular & Compatibility Features
* `widgets` - Ratatui widgets (`AccentGauge`, `AccentList`, `AccentTabs`, etc.).
* `effects` - ASCII particle systems (`MatrixRain`, `SimpleParticles`, `TuiEffect` trait).
* `sys-info` - System info helper parsing.
* `reg` - Config storage access / registry abstraction.
* `rgb` - OpenRGB protocol and controller handlers.
* `gpu` - Headless GPU compute context initialization and running utilities.

---

## 🚀 Usage

### 🩺 Diagnostic Doctor
All consumer applications can easily implement diagnostic modes by executing the doctor helper:

```rust
use rcommon::interface::cli::run_doctor;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--doctor") {
        run_doctor();
        return;
    }
}
```

### 🎨 Rendering TUI Effects

All effects follow a **Verb × Noun × Style × Palette** naming system. Pick an effect, then customize its style and palette:

```rust
use rcommon::interface::tui::effects::{
    FallingGlyphs, FallingDroplets, RisingFlames, FlowingParticles, PulledParticles,
    FallingComets, PulsingGlyphs, PulsingWaves,
    Style, Palette, TuiEffect,
};

// Default look (matrix-green with trails)
let mut rain = FallingGlyphs::new(80, 24, 0.35);

// Customize the render treatment and color
let mut flare_rain = FallingGlyphs::new(80, 24, 0.35)
    .with_style(Style::Flared)       // lens-flare heads
    .with_palette(Palette::Accent); // theme-matched color

// Game/render loop:
flare_rain.update(0.016, 80, 24);
flare_rain.draw(&mut grid, 80, 24);
```

### Dimensions

| Dimension | Values | What it controls |
|---|---|---|
| **Verb** | `Falling`, `Rising`, `Flowing`, `Pulled`, `Pulsing` | Motion model |
| **Noun** | `Glyphs`, `Particles`, `Droplets`, `Comets`, `Blocks`, `Waves` | Visual unit |
| **Style** | `Solid`, `Trailing`, `Flared` | Render treatment |
| **Palette** | `Monochrome(r,g,b)`, `Accent`, `Heat` | Color source |
| **Speed** | `Slow`, `Normal`, `Fast`, `Custom(u8)` | Velocity multiplier (0.25x-5.0x) |

### Built-in Effects (10)

| Type | Default style | Default palette | Best for |
|---|---|---|---|
| `FallingGlyphs` | `Trailing` | `Monochrome(Green)` | Matrix rain, code streams |
| `FlowingParticles` | `Solid` | `Monochrome(White)` | Ambient dust, snowfall |
| `PulledParticles` | `Solid` | `Monochrome(Blue)` | Gravity to center, magnets |
| `FallingDroplets` | `Solid` | `Monochrome(Blue)` | Rain, leaks, tears |
| `RisingFlames` | `Solid` | `Heat` | Fire, heat plumes |
| `FallingComets` | `Trailing` | `Monochrome(White)` | Shooting stars, magic trails |
| `PulsingGlyphs` | `Solid` | `Accent` | Heartbeat, focus indicator |
| `PulsingWaves` | `Solid` | `Heat` | Audio visualizer, ambient backdrop |
| `FlowingBlocks` | `Solid` | `Accent` | Tetris pieces drifting, retro gameplay |
| `PulledBlocks` | `Solid` | `Monochrome(Blue)` | Block particles to gravity center |

```rust
use rcommon::interface::tui::effects::{
    FallingDroplets, Speed,
};

let mut rain = FallingDroplets::new(80, 24)
    .with_speed(Speed::Fast); // 2x velocity
```

### How to Pick an Effect

1. **What's it doing?** Falling rain, rising smoke, pulsing heartbeat? → **Verb**
2. **What's it made of?** Characters, dots, streaks, shapes? → **Noun**
3. **How does it look?** Flat, with trail, with flare? → **Style**
4. **What color?** Fixed, theme-matched, temperature? → **Palette**

Combine any of the 5 verbs × 6 nouns = 30 base combinations. Each accepts any of the 3 styles × 3 palettes = 9 look variants, so 270 total — most are nonsense, but the matrix is the catalog.

---

## 🔄 Migration Guide (Legacy to Taxonomy)

> [!NOTE]
> **Taxonomy Features (Cargo features)** control what code is compiled in your `Cargo.toml` dependencies, whereas **Taxonomy Paths (module paths)** are the new Rust import locations in your code.

If you are migrating an older application to `rCommon` version 3.0.0+, update your module references as follows:

| Old Path | New Taxonomy Path |
| :--- | :--- |
| `rcommon::widgets::*` | `rcommon::interface::tui::widgets::*` |
| `rcommon::effects::*` | `rcommon::interface::tui::effects::*` |
| `rcommon::window::*` | `rcommon::lifecycle::foreground::window::*` |
| `rcommon::guard::*` | `rcommon::lifecycle::foreground::guard::*` |
| `rcommon::service::*` | `rcommon::lifecycle::background::service::*` |
| `rcommon::sys_info::*` | `rcommon::platform::native::sys_info::*` |
| `rcommon::reg::*` | `rcommon::platform::native::reg::*` |
| `rcommon::rgb::*` | `rcommon::role::application::rgb::*` |
| `rcommon::game::BhopGame` | `rcommon::role::application::game::ObstacleJumpGame` |

> **Note (3.1.0+):** The 5 TUI effects were renamed in 3.1.0 to follow the Verb+Noun taxonomy. Old type names (`MatrixRain`, `SimpleParticles`, `GravityParticles`, `RainEffect`, `FireEffect`) remain available as deprecated type aliases at the crate root. They will be removed in 4.0.0.
>
> ```rust
> // Old (still works, deprecated):
> use rcommon::MatrixRain;
>
> // New (preferred):
> use rcommon::interface::tui::effects::FallingGlyphs;
> ```

---

## 🎨 Visual Standards

All applications in the ecosystem share a cohesive user interface style and visual asset layout (e.g. icon containers, neon wireframe elements). See [docs/VISUAL_STANDARDS.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/rCommon/docs/VISUAL_STANDARDS.md) for details on visual guidelines and branding asset packaging.

## 📚 Embedding markdown docs (F1–F7 in-TUI help)

The `rcommon::embedded_docs!` macro bakes a set of markdown files (README, LICENSE, CONTRIBUTING, etc.) directly into your binary at compile time, so your TUI can show help text without reading the filesystem at runtime (which would break in a single-file `.scr` Windows screensaver install). See [docs/EMBEDDED_DOCS.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/rCommon/docs/EMBEDDED_DOCS.md) for the canonical example and the F1–F7 wiring pattern. (This pattern originated in rTemplate; the doc is now in rcommon so the 5 other r* TUI apps have a single source of truth.)

---

## 📄 License

This project is licensed under the MIT License. See [LICENSE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/rCommon/LICENSE.md) for details.
