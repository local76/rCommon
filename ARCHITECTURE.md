# library — Architecture

> The 4-layer taxonomy, the design system, the Screensaver trait, and the 10 screensaver scenes that form the foundation of every tool in the local76 ecosystem.

`library` is the shared Cargo crate that every other local76 tool depends on. This document explains how the code is organized, why it is organized that way, and how to add new code without breaking the contract.

---

## 1. Design principles

- **One crate, one foundation.** No other local76 repo re-implements sysinfo reads, registry helpers, Screensaver trait implementations, or the design-system widgets. If a piece of code will be needed by more than one consumer, it lives in `library`.
- **Classification by taxonomy.** Code is organized by the 4-layer model in §2. A change in one layer must not silently break a concern in another layer. The `tests/taxonomy_compliance.rs` test enforces this with an AST walker.
- **Absolute isolation between apps.** Each local76 app must run independently and must not interfere with the others. Three concrete rules:
  - **No global port binding.** No HTTP server on `127.0.0.1:8080`, no shared TCP port, no shared UDP port. Use local IPC: Unix domain sockets on Linux, Named Pipes on Windows.
  - **Per-app config storage.** Each app's config file lives at `%APPDATA%\<app>\config.yaml` (Windows) or `~/.config/<app>/config.yaml` (Linux). Registry access uses `Software\local76\<app>\*` paths. The app name is the namespace.
  - **Executable-scoped guards.** Single-instance guards and mutex locks are scoped to the executable's name, not the library's. `SingleInstanceGuard` for `helm` does not lock out `pulse`.

---

## 2. The 4-layer taxonomy

```
                ┌─────────────────────────────────────────────────────┐
                │  core  (the only mandatory layer;  no deps)        │
                │                                                     │
                │  • Screensaver trait, TerminalCell,                 │
                │    ScreenPalette, hsl_to_rgb, render_logo           │
                └─────────────────────┬───────────────────────────────┘
                                      │
       ┌──────────────────────────────┼──────────────────────────────┐
       │                              │                              │
       ▼                              ▼                              ▼
   interface                       lifecycle                     platform
   (Presentation)                (Execution State)            (Deployment)
   ─────────────────────         ─────────────────────         ─────────────────────
   • interface::tui              • lifecycle::foreground       • platform::native
     (ratatui backend,            (CLI entry, panic, TUI,         (sysinfo, winreg,
      widgets, effects)            window, drag-to-move,         OpenRGB, WiFi,
   • interface::cli                single-instance guard)        X11, DWM)
     (clap, doctor, scaffold)  • lifecycle::background       • platform::embedded
   • interface::api                (daemon, file_log,             (no-std)
     (IPC: messages,                service, event_log,         • platform::mobile
      win32_ipc, unix_ipc)          clipboard, notification)    • platform::web
   • interface::gui                                                (wasm)
     (egui helpers)
                                  role ─┐
                                         │  (which app is this?)
                                         ▼
                                       role
                                       (Purpose)
                                       ───────────
                                       • role::system
                                         (sys_info, reg, service)
                                       • role::application
                                         (palette, rgb, packages,
                                          10 screensaver scenes)
```

**`core` is the only layer that must remain neutral and usable by any combination of the other four.** It cannot depend on `interface`, `lifecycle`, `platform`, or `role`. Everything else can depend on `core` and on each other in any direction.

### Layer responsibilities

#### 2.1 `core` — neutral foundation
- `Screensaver` trait (backend-agnostic, depended on by every scene)
- `TerminalCell` (a single grid cell, ratatui-free)
- `ScreenPalette` (the cross-renderer color story, 7 RGB tuples)
- `LcgRng` (deterministic RNG for reproducible effects)
- `DashboardInfo`, `SystemInfo` (rich live system context)
- `render_logo_block`, `render_logo_5x5`
- `hsl_to_rgb`, `hsv_to_rgb`

#### 2.2 `interface` — presentation
- `interface::tui` — Ratatui backend, widgets (AccentGauge, AccentList, AccentTabs), the 10 canonical TUI effects (FallingGlyphs, RisingFlames, etc.), the `design::*` façade.
- `interface::cli` — clap, `run_doctor`, scaffold helpers.
- `interface::api` — IPC: messages, win32_ipc, unix_ipc.
- `interface::gui` — egui helpers (for the `platform::embedded` future).

#### 2.3 `lifecycle` — execution state
- `lifecycle::foreground` — `BorderlessConsole`, `SingleInstanceGuard`, `ConsoleTitleGuard`, `hide_console_at_startup`, `relaunch_in_conhost`, drag-to-move.
- `lifecycle::background` — daemon, service, file_log, event_log, clipboard, notification.

#### 2.4 `platform` — deployment
- `platform::native` — Windows + Linux FFI via `#[cfg(target_os = "...")]`. `sys_info`, `reg`, `monitors`, `power`, `dark_mode`, `dwm_accent`, `console_dpi`.
- `platform::embedded` (future) — no-std, routers, microcontrollers.
- `platform::mobile` (future) — iOS / Android.
- `platform::web` (future) — wasm.

#### 2.5 `role` — purpose
- `role::system` — Infrastructure: low-level registry, power, services, event logs, disk/BIOS queries.
- `role::application` — Task-oriented: `palette`, `rgb` (OpenRGB protocol), `packages` (winget), the 10 screensaver scenes, `game` (ObstacleJump).

---

## 3. The 4.0 design system

`library` 4.0 introduced a single import path for every chrome concern: `library::interface::tui::design::prelude::*`. Before 4.0, the same chrome types (`StatusBar`, `MarkdownViewerState`, `ThemeColors`, `AccentColors`, ...) were scattered across `library::interface::tui::*`, `library::widgets::*`, and the consumer apps' own `win32.rs` shims. Each r* app re-implemented its own `is_dark_mode()` registry read and its own HSL accent-rotation math. The result was visual drift between `helm`, `pulse`, and `trance`, and a lot of code duplication.

The 4.0 design system fixes this in three moves:

### 3.1 One façade

```rust
use library::interface::tui::design::prelude::*;
```

This brings in the entire visual identity: theme, accent bundles, status bar, toast, markdown viewer, layout guard, title banner, effect preview, mouse selection, layout helpers, text utilities, terminal-size constants, all 10 canonical TUI effects, and the unified `Screensaver` trait. See [docs/DESIGN_SYSTEM.md](docs/DESIGN_SYSTEM.md) for the full onboarding guide.

### 3.2 One palette

`library::role::application::palette::ScreenPalette` is the canonical color story:

```rust
pub struct ScreenPalette {
    pub bg:     (u8, u8, u8),
    pub fg:     (u8, u8, u8),
    pub accent: (u8, u8, u8),
    pub dim:    (u8, u8, u8),  // 35% of accent
    pub hot:    (u8, u8, u8),  // accent hue +30°
    pub cool:   (u8, u8, u8),  // accent hue -120°
    pub mid:    (u8, u8, u8),  // neutral chrome
    pub peak:   (u8, u8, u8),  // white-hot peaks
}
```

The same RGB tuples drive both Ratatui TUI chrome and GDI pixel renderers. `query_current_palette()` is the cross-platform helper that returns one. The TUI-side `dimensions::Palette` enum exposes `Accent`, `AccentDim`, `AccentHot`, `AccentCool` variants that map 1:1 onto `ScreenPalette`'s fields.

### 3.3 One `Screensaver` trait

```rust
pub trait Screensaver {
    fn init(&mut self, _cols: usize, _rows: usize) {}
    fn update(&mut self, dt: Duration, cols: usize, rows: usize);
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize);
    fn has_scanlines(&self) -> bool { false }
}
```

The trait lives in `core` because it depends only on `TerminalCell` (also in `core`) and `std::time::Duration`. Both TUI effects and GDI screensavers can implement it. The pre-4.0 Ratatui-coupled trait is a thin wrapper at `library::interface::tui::screensaver::ScreensaverRenderer` for the buffer-management concerns only.

### 3.4 The `design/` subfolder

```
src/interface/tui/design/
├── mod.rs                façade + prelude + re-exports
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

The pre-4.0 module paths (`library::interface::tui::theme`, `library::interface::tui::markdown`, `library::interface::tui::status`, `library::interface::tui::layout`, `library::interface::tui::text`, `library::widgets::colors`, ...) are kept as deprecated re-exports for one minor release (4.0 → 4.1). They will be removed in 4.1.

---

## 4. The 10 screensaver scenes

All 10 scene implementations live at `library::role::application::scenes::<name>`. The binary shims in the [`screensavers`](https://github.com/local76/screensavers) workspace are 20-line wrappers around them.

| Module | Type | What it is |
|---|---|---|
| `scenes::beams` | `Beams` | 4 colored spotlight cones sweeping over a starfield with rising dust + lens flares. Centered system logo. |
| `scenes::bounce` | `BhopDashboard` | 3-panel cyberpunk TUI dashboard: live system info, fake command console, bunny-hop mini-game. |
| `scenes::flame` | `FireEffect` | Bottom-up cellular-automaton fire, Doom-style. Centered system logo warms with the fire. |
| `scenes::gnats` | `Fireflies` | 30–60 fireflies in a triadic palette. Boid-style predator/prey. Wireframe network. Starfield. |
| `scenes::bursts` | `Fireworks` | City skyline at bottom. Rockets launch and explode into colored particle bursts. Windows light up. |
| `scenes::cosmos` | `LifeEffect` | Full universe lifecycle: Darkness → BigBang → Expansion → Accretion → Singularity → Collapse. |
| `scenes::glyphs` | `Glyphs` | Falling katakana+digit rain. Head of each stream draws from the host's hostname+OS+kernel. |
| `scenes::disco` | `Party` | Disco ball + 8 sweeping rays + neon confetti + VU-meter equalizer + audio beat. |
| `scenes::storm` | `Pour` | Cold rain over mountains + pine forest. Periodic lightning. Bird on a tree. Deer/bear/bigfoot walk by. |
| `scenes::chaos` | `Unstable` | System logo disassembles. 7 chaos types: Supernova, BlackHole, Vortex, GlitchWave, Shockwave, Entropy, Resonance. RGB chromatic-aberration glitch + spring snap-back. |

Each scene exposes a `new()` constructor (no args) and a `name()` method returning the lowercase scene name. Each implements the `Screensaver` trait (init, update, draw, has_scanlines). `ScreensaverRenderer` is the TUI-side buffer manager; the GDI / raw-termios loop in `library::screensaver_runtime` is what the `screensavers` workspace's shim binaries call.

---

## 5. TUI effect naming — Verb × Noun × Style × Palette

The 10 effects in `library::interface::tui::effects` follow a 4-dimension naming system. The type name is always `Verb` + `Noun` (PascalCase); the file name is the snake_case of the same; the display name is `"Verb Noun"`.

| Dimension | Values | Purpose |
|---|---|---|
| **Verb** | `Falling`, `Rising`, `Flowing`, `Pulled`, `Pulsing` | Motion model |
| **Noun** | `Glyphs`, `Particles`, `Droplets`, `Comets`, `Blocks`, `Waves` | Visual unit |
| **Style** | `Solid`, `Trailing`, `Flared` | Render treatment |
| **Palette** | `Monochrome(r,g,b)`, `Accent`, `Heat`, `AccentDim`, `AccentHot`, `AccentCool` | Color source |

- **Style** lives in `interface::tui::effects::dimensions::Style` and is exposed as a field on every effect.
- **Palette** lives in `interface::tui::effects::dimensions::Palette` and is exposed as a field on every effect.
- All effects expose `with_style(Style)` and `with_palette(Palette)` builder methods.
- The 4 dimensions are mutually orthogonal. 5 verbs × 6 nouns = 30 base combinations × 6 palettes = 180; with 3 styles = 540 total. Most are nonsense, but the matrix is the catalog.

### House rules

- **Adding a new Verb** requires a CHANGELOG entry justifying that it cannot be expressed as a variant of an existing verb.
- **Adding a new Noun** requires a CHANGELOG entry justifying that it cannot be expressed as a variant of an existing noun.
- **Adding a new Style** must be visually distinct (not a color or timing tweak).
- **Adding a new Palette** must be non-trivial (not just `Monochrome` with math).
- **Hard caps**: 5 verbs, 6 nouns, 3 styles, 6 palettes. Adding past a cap requires a documented justification.

### Current catalog (10 effects)

| Type | File | Default Style | Default Palette |
|---|---|---|---|
| `FallingGlyphs` | `falling_glyphs.rs` | `Trailing` | `Monochrome(Green)` |
| `FlowingParticles` | `flowing_particles.rs` | `Solid` | `Monochrome(White)` |
| `PulledParticles` | `pulled_particles.rs` | `Solid` | `Monochrome(Blue)` |
| `FallingDroplets` | `falling_droplets.rs` | `Solid` | `Monochrome(Blue)` |
| `RisingFlames` | `rising_flames.rs` | `Solid` | `Heat` |
| `FallingComets` | `falling_comets.rs` | `Trailing` | `Monochrome(White)` |
| `PulsingGlyphs` | `pulsing_glyphs.rs` | `Solid` | `Accent` |
| `PulsingWaves` | `pulsing_waves.rs` | `Solid` | `Heat` |
| `FlowingBlocks` | `flowing_blocks.rs` | `Solid` | `Accent` |
| `PulledBlocks` | `pulled_blocks.rs` | `Solid` | `Monochrome(Blue)` |

---

## 6. Adding new code

1. **Classify using the taxonomy.** Which of the 4 layers does this code belong to? Which `role`? If it is neutral and reusable, it goes in `core`. If it depends on a TUI, an OS, a service, or a task, it goes in the matching submodule.
2. **Place in the matching module.** Use `core` only for truly neutral data.
3. **Add documentation with a classification comment.** A 1-line `// Classification: <layer>` at the top of the file.
4. **Gate behind the appropriate Cargo feature.** `interface-tui` for TUI, `platform-native` for OS-specific FFI, `role-application` for app-level task code, etc.
5. **Update this `ARCHITECTURE.md` and the relevant `mod.rs` docs.** New code that fits an existing pattern doesn't need a new section here; a new pattern does.
6. **Provide cross-platform stubs where possible.** A `platform::native::monitors::get_monitors_summary` on Linux should return the Xinerama / XRandR result, not a Windows-only stub.
7. **Avoid putting presentation / lifecycle / platform code into `core`.** The `tests/taxonomy_compliance.rs` AST walker will fail the test if you do.

---

## 7. Taxonomy compliance (the test)

`tests/taxonomy_compliance.rs` walks the AST and fails the build if:

- A `core/*` file imports from `interface/`, `lifecycle/`, `platform/`, or `role/`.
- A `design/*` file imports from `lifecycle/`, `platform/`, or `role/`.
- A `platform::embedded/*` file imports from `interface::gui`, `interface::tui`, or any `platform::native/*`.

This catches the most common mistake: a `core` type that depends on a TUI backend, which then traps a `lifecycle::background` service that tries to use it without enabling the TUI feature.

---

## 8. Windows / Linux / GitHub

- **Windows / Linux**: Primarily `platform::native` with strong influence on `lifecycle` (services vs daemons, conhost behavior) and `role::system` (registry vs config files, DWM, power APIs). Code uses `#[cfg(target_os = "...")]` and per-platform splits (e.g., `platform/native/sys_info/windows.rs`).
- **GitHub**: External to the runtime taxonomy. Supports `platform::native` distribution via git dependencies and tagged releases. Hosts the org profile and packaging metadata. The monorepo-like local setup (`toolkit/scripts/build.ps1` and the `[patch]` redirect in every consumer's `Cargo.toml`) relies on it.

---

## 9. 4.0 breaking changes summary

| Change | Migration |
|---|---|
| `Screensaver` trait moved from `interface::tui::screensaver` to `core::screensaver` | Update import path; trait shape identical |
| `Screensaver::update` takes `Duration` (was `f32`) | `Duration::from_secs_f32(dt)` to bridge |
| `ScreensaverRenderer::tick` (3.x, `f32`) → `tick_duration` (4.0, `Duration`) | Rename call sites; old method is a deprecated shim |
| `Screensaver` methods declared directly on the trait (no separate `ScreensaverState` / `Effect` supertraits) | Library effects: split `impl ScreensaverState` + `impl ScreensaverEffect` into `impl Screensaver` directly |
| New `Screensaver::has_scanlines` (default `false`) | Effects that want scanlines opt in |
| `TerminalCell::draw` takes `&self` (was `&mut self`) | Effects that mutated state in `draw` use `RefCell` (library has 2; `screensavers` has 0) |
| `ScreensaverEffect` trait re-exported as deprecated trait alias | One minor of warnings; will be removed in 4.1 |
| Module split into `design/` subfolder | 3.x paths re-exported as deprecated module aliases |
| New `ScreenPalette` and `query_current_palette()` | Apps replace hand-rolled HSL math + registry reads |
| `dimensions::Palette` gains `AccentDim`, `AccentHot`, `AccentCool` variants | TUI effects opt in; old `Monochrome / Accent / Heat` unchanged |
| Version bump 3.4.4 → 4.0.0 | Update consumer `Cargo.toml` to require `library = "4.0"` (or `library = { git = "...", tag = "v4.0.0" }`) |

---

## 10. Consumers

- [`screensavers`](https://github.com/local76/screensavers) — the 10 binary shims, each a 20-line wrapper around a `scenes::*` type.
- [`trance`](https://github.com/local76/trance) — Windows screensaver host + TUI picker. Uses `interface::tui::design::prelude::*` for the picker chrome.
- [`helm`](https://github.com/local76/helm) — system info dashboard. Uses `interface::tui` for the dashboard, `platform::native::sys_info` for the data.
- [`pulse`](https://github.com/local76/pulse) — live resource monitor. Uses `interface::tui` and `platform::native::sys_info`.
- [`scout`](https://github.com/local76/scout) — WiFi scanner. Uses `interface::tui` and `platform::native::wifi`.
- [`ignite`](https://github.com/local76/ignite) — startup-time dashboard. Uses `interface::tui`, `lifecycle::foreground::config`, `platform::native::reg`.

All 6 consumers depend on `library` via the `[patch."https://github.com/local76/library.git"]` redirect. See [`README.md`](README.md#add-as-a-dependency) for the full dependency story.

---

## 11. Future

- Full port of reusable effects from the old `trance-scenes` and `r*` days is now complete (all 10 scenes live in `scenes::*`).
- Stronger API / headless support (the `interface::api` and `lifecycle::background::service` modules).
- Better CLI vs TUI separation (the `interface::cli` façade).
- CI enforcement of taxonomy beyond the local `tests/taxonomy_compliance.rs` test.
- Potential workspace split per major section (`library-interface-tui` as a separate crate).
