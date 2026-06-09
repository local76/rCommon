# library Architecture

library is the shared foundation library for the local76 "r*" ecosystem of local-first terminal and system utilities (trance, helm, pulse, template, scout, ignite, trance-scenes, etc.).

## Design Principles

- **Avoid duplication**: Common utilities, platform abstractions, TUI widgets, effects, and system info live here so individual apps stay small and focused.
- **Classification by Taxonomy**: Code is organized according to the 4-layer taxonomy to prevent accidental coupling between concerns (e.g., a TUI-only effect type being changed in a way that breaks a background service or CLI tool).
- **Absolute Isolation & Non-Interference**: All `apps` must work independently on their own and not interfere with one another. To guarantee this:
  - **No Global Port Binding**: Do not bind to shared network ports (like HTTP servers on `127.0.0.1:8080`) for communication or single-instance locks. This prevents firewall prompts and port collisions. Use application-scoped Local IPC (Unix domain sockets on Linux and Named Pipes on Windows) instead.
  - **Isolated Configuration Storage**: Ensure configuration schemas and storage access (like Registry paths or configuration files) are strictly namespaced under the application name (`Software\apps\<AppName>`) so they never overwrite each other.
  - **Executable-Scoped Guards**: Single-instance guards and Mutex locks must be scoped using the executable's name to avoid multi-instance conflicts without locking out other `apps`.


### 1. Interface (Presentation Layer)
How the software communicates visually (or non-visually) with the user or other software.
- **CLI** (Command Line Interface)
- **TUI** (Text User Interface) — ratatui-based, TerminalCell grids, effects, widgets, logo rendering.
- **GUI: Native/OS** — Standard WIMP using OS toolkit.
- **GUI: Custom/Game Engine** — Continuous-loop canvas (e.g., egui, custom renderers).
- **Headless / API** — No UI; communicates with other software (REST, IPC, libraries, daemons exposing APIs).

#### Key Distinctions
- **CLI**: Line-oriented stdio, sequential output, often stateless per invocation.
- **TUI**: Grid-based, in-place updates, stateful widgets/navigation, keyboard-driven.
- **GUI**: Pixel canvas, continuous 2D space, rich mouse input, higher visual fidelity.

### 2. Execution State (Lifecycle)
How the OS manages the application's runtime.
- **Foreground Applications** — Require active user attention, window/terminal focus (console guards, hiding, single-instance, title management).
- **Background Processes** — Run silently, often at startup, no interface (services, event logging, power management for daemons).

### 3. Platform & Architecture (Deployment)
How the software is packaged and where it runs.
- **Native Applications** — Compiled for host OS/hardware (primary focus: Windows + Linux via cfg(target_os)).
- **Web Applications** — Browser (stubs/future).
- **Mobile Applications** — iOS/Android (future).
- **Embedded Software** — Dedicated hardware (routers, etc., future).

Windows and Linux specifics (FFI, services, registry emulation, console behavior, power, monitors, etc.) live primarily here, with platform splits (e.g., `platform/native/windows.rs`).

GitHub fits as the **distribution and collaboration mechanism** for the Native + git-dependency model used across the suite (e.g., `library = { git = "https://github.com/local76/library.git" }` + `[patch]` for local dev). It is not a runtime concern but enables the ecosystem.

### 4. System Role (Purpose)
The software's ultimate objective.
- **System Software** — Infrastructure: manages hardware, provides platform (low-level power, registry, services, event logs, disk enumeration, BIOS, shell detection).
- **Application Software** — Task-oriented for end-users or tools (RGB control, games/effects, package inventory, TUI dashboards, higher-level formatting).

## Current Module Structure (Aligned to Taxonomy)

- **core/** (or core.rs): Neutral, cross-cutting primitives usable by *any* combination of the above. Must remain free of heavy UI, OS, or lifecycle assumptions.
  - LcgRng (deterministic RNG for effects/games)
  - TerminalCell (universal for grid renderers: TUI effects, headless logging, custom GUIs)
  - DashboardInfo / SystemInfo (rich live system context with logo_text, uptime, mem, power, etc.)
  - get_dashboard_info, get_packages_breakdown, get_monitors_summary, render_logo_block, etc.
  - Classification: Core foundation for all layers.

- **interface/**
  - **tui/**: TUI-specific presentation (widgets like AccentGauge, effects like MatrixRain/ParticleSystem, text wrapping/alignment helpers, ObstacleJump game rendering, logo block).
    - Classification: Interface (TUI) + Role (Application) + some Platform.
  - **gui/**: GUI helpers (eframe/egui for custom/game-engine UIs).
    - Classification: Interface (GUI Custom).
  - **api/**: Headless/API surfaces (IPC, service exposure, library interfaces).
    - Classification: Interface (Headless/API).
  - Future: cli/, gui_native/.

- **lifecycle/**
  - **foreground/**: Console/window management for apps that need focus (BorderlessConsole, SingleInstanceGuard, ConsoleTitleGuard, hide_console_at_startup, relaunch_in_conhost).
    - Classification: Lifecycle (Foreground) + Platform (Native).
  - **background/**: Silent running (services, event logging, notifications, clipboard for daemons, thread execution state).
    - Classification: Lifecycle (Background) + Role (System).

- **platform/**
  - **native/**: OS-specific deployment (sys_info with Windows/Linux splits, reg, monitor enumeration, power, dark mode, accent via DWM/registry, console DPI, etc.).
    - Classification: Platform (Native) + Role (System).
  - Future: web/, mobile/, embedded/.

- **role/**
  - **system/**: Infrastructure (low-level registry, power, services, event logs, disk/BIOS queries).
    - Classification: Role (System Software).
  - **application/**: Higher-level/task-oriented (RGB controller, games/effects integration, package inventory, formatting helpers like get_battery_info).
    - Classification: Role (Application Software).

- **Other**:
  - `rgb/`: RGB lighting control (OpenRGB protocol + controller). Classification: Role (Application) + Interface (for effects).
  - `bin/rpack.rs`: Packaging tool (builds, deb/rpm, etc.). Classification: Tooling / Role (Application).
  - Legacy `win32` shim (deprecated, for old consumers).

## Windows OS, Linux OS, and GitHub

- **Windows/Linux**: Primarily **Platform (Native)** with strong influence on **Lifecycle** (services vs daemons, conhost behavior) and **Role (System)** (registry vs config files, DWM, power APIs). Code uses `#[cfg(target_os = "...")]` and splits (e.g., `platform/native/windows.rs`, `sys_info/windows.rs`).
- **GitHub**: External to runtime taxonomy. Supports **Platform (Native)** distribution via git dependencies and releases. Also hosts org profile, workflows, and packaging metadata. The monorepo-like local setup (git_push_all, patches) relies on it.

## Effect Naming Taxonomy (Verb × Noun × Style × Palette)

All TUI effects in `interface::tui::effects` follow a 4-dimension naming system. The type name is always **`Verb` + `Noun`** (PascalCase), the file name is the snake_case of the same, and the display name is `"Verb Noun"`.

### Dimensions

| Dimension | Values | Purpose |
|---|---|---|
| **Verb** | `Falling`, `Rising`, `Flowing`, `Pulled`, `Pulsing` | Motion model |
| **Noun** | `Glyphs`, `Particles`, `Droplets`, `Comets`, `Blocks`, `Waves` | Visual unit |
| **Style** | `Solid`, `Trailing`, `Flared` | Render treatment |
| **Palette** | `Monochrome(r,g,b)`, `Accent`, `Heat` | Color source |

- **Style** lives in `interface::tui::effects::dimensions::Style` and is exposed as a field on every effect.
- **Palette** lives in `interface::tui::effects::dimensions::Palette` and is exposed as a field on every effect.
- All effects expose `with_style(Style)` and `with_palette(Palette)` builder methods.
- The 4 dimensions are mutually orthogonal and any combination is valid (270 total, ~100 meaningful).

### House Rules

- **Adding a new Verb** requires a CHANGELOG entry justifying that it cannot be expressed as a variant of an existing verb.
- **Adding a new Noun** requires a CHANGELOG entry justifying that it cannot be expressed as a variant of an existing noun.
- **Adding a new Style** must be visually distinct (not a color or timing tweak).
- **Adding a new Palette** must be non-trivial (not just `Monochrome` with math).
- **Hard caps**: 5 verbs, 6 nouns, 3 styles, 3 palettes. Adding past a cap requires a documented justification.

### Current Catalog (5 effects)

| Type | File | Default Style | Default Palette |
|---|---|---|---|
| `FallingGlyphs` | `falling_glyphs.rs` | `Trailing` | `Monochrome(Green)` |
| `FlowingParticles` | `flowing_particles.rs` | `Solid` | `Monochrome(White)` |
| `PulledParticles` | `pulled_particles.rs` | `Solid` | `Monochrome(Blue)` |
| `FallingDroplets` | `falling_droplets.rs` | `Solid` | `Monochrome(Blue)` |
| `RisingFlames` | `rising_flames.rs` | `Solid` | `Heat` |

### Example Usage

```rust
use library::interface::tui::effects::{
    FallingGlyphs, Style, Palette,
};

// Default: matrix-green with trails
let mut rain = FallingGlyphs::new(80, 24, 0.35);

// Theme-matched with lens-flare heads
let mut flare = FallingGlyphs::new(80, 24, 0.35)
    .with_style(Style::Flared)
    .with_palette(Palette::Accent);
```

## Adding New Code

1. Classify using the taxonomy.
2. Place in the matching module (use `core` only for truly neutral data).
3. Add documentation with classification comment.
4. Gate behind appropriate feature (e.g., `effects` for TUI visuals, `sys-info` for platform queries).
5. Update this ARCHITECTURE.md and relevant mod.rs docs.
6. Provide cross-platform stubs where possible.
7. Avoid putting presentation/lifecycle code into `core`.

## Current State & Audit Notes

The structure has been cleaned up and aligned (see list of 10 tasks executed in the refactor session). Old flat files were moved into taxonomy categories. Re-exports preserve compat for r* consumers using git + [patch].

**Audit of Ports from Other Projects** (valuable reusable pieces extracted and classified):
- From trance-scenes/trance-core: SystemInfo/get_system_info (core + platform), render_logo_block + 5x5 (interface/tui), LcgRng enhancements (core), registry/theme helpers (already via core/platform).
- From trance-scenes effects (rObstacleJump, cosmos, glyphs, etc.): Particle systems, MatrixRain, ObstacleJump logic (interface/tui + role/application), console typing/dashboard patterns.
- From helm: Package counting (role/application/packages.rs), monitor enumeration (platform/native/monitors.rs), accent/dark mode/power formatters (platform + lifecycle).
- From trance (saver_win32): Advanced console (high contrast, thread exec state, titles, screensaver control) (lifecycle/foreground/console.rs), power/accent delegation.
- From pulse/ignite/template/scout: Common win32 shims, console hiding, system queries (now centralized in lifecycle/platform/role).
- rpack bin and rgb/game already in library (role/application).

## Migration Guide for Consumers

To ensure long-term architecture sustainability, consumers should move away from the deprecated `library::win32` module (legacy flat shim) and transition to the new 4-layer taxonomy modules.

> [!NOTE]
> **Taxonomy Features (Cargo features)** control what code is compiled in your `Cargo.toml` dependencies, whereas **Taxonomy Paths (module paths)** are the new Rust import locations in your code.

Here is a concrete "Before & After" mapping for imports and usage:

### 1. Presentation Layer (Interface)
* **TUI Effects / Primitives**:
  * *Before*: `use library::win32::{TerminalCell, MatrixRain, SimpleParticles};`
  * *After*: `use library::interface::tui::effects::{TerminalCell, MatrixRain, SimpleParticles};` (or `library::core::TerminalCell` / `library::interface::tui::MatrixRain`)
* **TUI Focus Widgets**:
  * *Before*: `use library::widgets::{AccentList, AccentTabs};`
  * *After*: `use library::interface::tui::widgets::{AccentList, AccentTabs};`
* **Headless IPC**:
  * *Before*: `use library::api::{IpcServer, IpcClient};`
  * *After*: `use library::interface::api::{IpcServer, IpcClient};`

### 2. Execution State Layer (Lifecycle)
* **Console & Window Controls**:
  * *Before*: `use library::win32::{hide_console_at_startup, BorderlessConsole, ConsoleTitleGuard};`
  * *After*: `use library::lifecycle::foreground::window::{hide_console_at_startup, BorderlessConsole, ConsoleTitleGuard};`
* **Single Instance Lock**:
  * *Before*: `use library::win32::SingleInstanceGuard;`
  * *After*: `use library::lifecycle::foreground::guard::SingleInstanceGuard;`
* **Services & Notifications**:
  * *Before*: `use library::win32::{query_windows_service_status, show_toast_notification};`
  * *After*: `use library::lifecycle::background::service::query_service_status;` and `use library::lifecycle::background::notification::show_toast_notification;`

### 3. Platform & Architecture Layer (Deployment)
* **System Theme & Uptime Helpers**:
  * *Before*: `use library::win32::{query_dark_mode, get_system_screen_resolution, get_dwm_accent_color};`
  * *After*: `use library::platform::native::sys_info::{query_dark_mode, get_system_screen_resolution, get_dwm_accent_color};`
* **Monitor / Screen Enumeration**:
  * *Before*: `use library::win32::{get_monitors_summary, get_all_monitors};`
  * *After*: `use library::platform::native::monitors::{get_monitors_summary, get_all_monitors};`

### 4. System Role Layer (Purpose)
* **Application Roles (RGB/Packages)**:
  * *Before*: `use library::win32::{get_packages_breakdown, RgbController};`
  * *After*: `use library::role::application::packages::get_packages_breakdown;` and `use library::role::application::rgb::controller::RgbController;`
* **Registry Access (Infrastructure)**:
  * *Before*: `use library::win32::{read_string, write_string, HKEY_CURRENT_USER};`
  * *After*: `use library::platform::native::reg::{read_string, write_string, HKEY_CURRENT_USER};`

This structure allows multiple crates per section in the future (e.g., library-interface-tui as a separate crate) while keeping the single-crate experience simple for git-based consumption in the r* apps.

## library 4.0 Design System

In library 4.0 every r* TUI app (helm, pulse, trance, template, scout, hub) and every r* GDI screensaver app (trance-scenes: cosmos, gnats, glyphs, flame, ...) consumes a **unified design system** from a single import path. The 4.0 release is the official v4.0.0 across library + all apps + the rScreensavers.

### Single import path

```rust
use library::interface::tui::design::prelude::*;
```

This brings in the entire visual identity: theme, accent bundles, status bar, toast, markdown viewer, layout guard, title banner, effect preview, mouse selection, layout helpers, text utilities, terminal-size constants, all 12 canonical TUI effects, and the unified `Screensaver` trait. See `docs/DESIGN_SYSTEM.md` for the full onboarding guide.

### Why a design system now?

Before 4.0, the same chrome types (`StatusBar`, `MarkdownViewerState`, `ThemeColors`, `AccentColors`, ...) were scattered across `library::interface::tui::*`, `library::widgets::*`, and the consumer apps' own `win32.rs` shims. Each r* app re-implemented its own `is_dark_mode()` registry read and its own HSL accent-rotation math. The result was visual drift between helm, pulse, and trance-scenes — and a lot of code duplication.

The 4.0 design system fixes this in three moves:

1. **One façade**: `library::interface::tui::design` is the single import surface. Everything an r* TUI needs lives there.
2. **One palette**: `library::role::application::palette::ScreenPalette` is the canonical color story. The same RGB tuples drive both ratatui TUI chrome and GDI pixel renderers. `query_current_palette()` is the cross-platform helper that returns one.
3. **One Screensaver trait**: `library::core::screensaver::Screensaver` (backend-agnostic, in `core` so it can be implemented by both TUI effects and GDI screensavers). The pre-4.0 ratatui-coupled trait is a thin wrapper at `library::interface::tui::screensaver::ScreensaverRenderer` for the buffer-management concerns only.

### 4.0 source-of-truth files (the `design/` subfolder)

In library 4.0 every chrome file lives under `src/interface/tui/design/`:

```
src/interface/tui/design/
├── mod.rs                Façade + prelude + re-exports
├── theme.rs              ThemeColors, get_theme, accent_color_from_hex
├── colors.rs             AccentColors, AccentTheme (3-color bundles)
├── status.rs             StatusBar (4-second decay pattern)
├── toast.rs              ToastBox, ToastKind
├── markdown.rs           parse_markdown_to_lines, draw_markdown_modal
├── markdown_viewer.rs    MarkdownViewerState (F1-F7 state machine)
├── layout_guard.rs       is_too_small, render_too_small_warning
├── title_banner.rs       draw_title_banner, ButtonRect
├── effect_preview.rs     draw_effect_preview
├── mouse_selection.rs    MouseSelection
├── layout.rs             centered_rect, format_help_row
└── text.rs               wrap_text, align_line, char_width, visible_len, ...
```

The 3.x module paths (`library::interface::tui::theme`, `library::interface::tui::markdown`, `library::interface::tui::status`, `library::interface::tui::layout`, `library::interface::tui::text`, `library::widgets::colors`, ...) are kept as **deprecated** re-exports for one minor release (4.0 → 4.1). They will be removed in 4.1.

### `Screensaver` trait moved to `core`

The 4.0 split: the trait is backend-agnostic, the renderer is TUI-only.

```
src/core/screensaver.rs
  pub trait Screensaver {
      fn init(&mut self, _cols: usize, _rows: usize) {}
      fn update(&mut self, dt: Duration, cols: usize, rows: usize);
      fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize);
      fn has_scanlines(&self) -> bool { false }
  }
  pub trait ScreensaverState { ... }   // active/focused sub-trait
  pub trait ScreensaverEffect { ... }   // deprecated alias for back-compat

src/interface/tui/screensaver.rs
  pub struct ScreensaverRenderer { ... }  // buffer management, TUI-only
```

**4.0 signature changes**:
- `update` now takes `Duration` (was `f32` seconds in 3.x). Use
  `Duration::from_secs_f32(dt)` to bridge.
- `ScreensaverRenderer::tick_duration` is the new Duration-based entry point. The pre-4.0 `tick(&mut s, f32)` is a deprecated shim.

### `ScreenPalette` for cross-renderer color story

```rust
pub struct ScreenPalette {
    pub bg: (u8, u8, u8),
    pub fg: (u8, u8, u8),
    pub accent: (u8, u8, u8),
    pub dim: (u8, u8, u8),       // 35% of accent
    pub hot: (u8, u8, u8),      // accent hue +30°
    pub cool: (u8, u8, u8),     // accent hue -120°
    pub mid: (u8, u8, u8),      // neutral chrome
    pub peak: (u8, u8, u8),     // white hot peaks
}
```

`ScreenPalette` is in `library::role::application::palette` (Application role, backend-agnostic — no ratatui types). Every r* app constructs one via `query_current_palette()` and reads the same fields.

The TUI-side `dimensions::Palette` enum exposes `ACCENT`, `ACCENT_DIM`, `ACCENT_HOT`, `ACCENT_COOL` variants that map 1:1 onto `ScreenPalette`'s accent/dim/hot/cool fields. This means r* TUI effects, r* GDI screensavers, and r* dashboards all derive the same color story from the same system accent.

### Taxonomy compliance

The 4-layer taxonomy (Core / Interface / Lifecycle / Platform / Role) is unchanged in 4.0. The design system files are strictly under **Interface (TUI / Presentation Layer)** — `design/*` does not import from `lifecycle/`, `platform/`, or `role/`. `ScreenPalette` is in **Role (Application Software)** because it is a task-level reusable. The unified `Screensaver` trait is in **Core** because it depends only on `TerminalCell` (also Core) and `std::time::Duration`.

The `tests/taxonomy_compliance.rs` AST-walker catches any future violation: a new `design/*` file that imports from `lifecycle/`, `platform/`, or `role/` will fail the test.

### 4.0 breaking changes summary

| Change | Migration |
|---|---|
| `Screensaver` trait moved from `interface::tui::screensaver` to `core::screensaver` | Update import path; trait shape identical |
| `Screensaver::update` takes `Duration` (was `f32`) | `Duration::from_secs_f32(dt)` to bridge |
| `ScreensaverRenderer::tick` (3.x, `f32`) → `tick_duration` (4.0, `Duration`) | Rename call sites; old method is deprecated shim |
| `Screensaver` methods `init/update/draw/has_scanlines` declared directly on the trait (no separate `ScreensaverState`/`Effect` supertraits) | library effects: split `impl ScreensaverState` + `impl ScreensaverEffect` into `impl Screensaver` directly |
| New `Screensaver::has_scanlines` (default `false`) | trance-scenes effects that want scanlines opt in |
| TerminalCell `pub fn draw(&self, ...)` (was `&mut self`) | library effects that mutated state in draw use `RefCell` (library has 2; trance-scenes has 0) |
| `ScreensaverEffect` trait re-exported as deprecated trait alias | One minor of warnings; will be removed in 4.1 |
| Module split into `design/` subfolder | 3.x paths re-exported as deprecated module aliases |
| New `ScreenPalette` and `query_current_palette()` | r* apps replace hand-rolled HSL math + registry reads |
| `dimensions::Palette` gains `AccentDim`, `AccentHot`, `AccentCool` variants | TUI effects opt in; old `Monochrome/Accent/Heat` unchanged |
| Version bump 3.4.4 → 4.0.0 | Update r* `Cargo.toml` to require `library = "4.0"` (or `library = { git = "...", tag = "v4.0.0" }`) |

## Related Projects (for context)

- trance / trance-scenes: Heavy use of TUI effects, lifecycle (screensavers as background/foreground), platform (console/windowing).
- helm / pulse / ignite: System info, package inventory, monitors, power, dark mode (Role + Platform + Interface TUI).
- template (incl. window subcrate): GUI custom + TUI, diagnostics.
- scout: Platform (WLAN), TUI, lifecycle.
- All use library via git + local [patch] for development.

## Future

- Full port of reusable effects from trance-scenes.
- Stronger API/Headless support.
- Better CLI vs TUI separation.
- CI enforcement of taxonomy (no cross-layer imports).
- Potential workspace split per major section.