# Changelog

All notable changes to library will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Release process (for maintainers)

```bash
# 1. From the library repo root, stage and commit the 4.0 changes
cd ../library
git add -A
git commit -m "library 4.0.0: unified design system + backend-agnostic Screensaver

- Add library::interface::tui::design (single import path for r* TUI apps)
- Move Screensaver trait to library::core (backend-agnostic, unified on Duration)
- Add library::role::application::palette::ScreenPalette
- Bridge ScreenPalette to dimensions::Palette (AccentDim/Hot/Cool variants)
- Add library::core::screensaver::ScreensaverState supertrait
- Move chrome files into design/ subfolder (deprecated re-exports for 4.0.x)
- 130 tests pass (96 unit + 11 design facade + 2 taxonomy + 21 doctests)"

# 2. Tag the release
git tag -a v4.0.0 -m "library 4.0.0 — Unified design system for the apps suite"
git push origin main --follow-tags

# 3. (Optional) Update each r* app to require library = \"4.0\" (instead of branch = \"main\")
#    so consumers outside the local [patch] tree get the stable release.
```

After tagging, all r* apps that use `[patch."https://github.com/local76/library.git"]`
will resolve to the 4.0.0 release once the local library directory is removed
or the patch entries are commented out.

## [4.2.0] - 2026-06-08

### Added
- **`library::screensaver_runtime` module**: New cross-platform screensaver host loop. `run_main(saver, name)` parses the standard screensaver CLI args (`/s` run, `/c` configure, `/p HWND` preview, `-h` help) and dispatches to the platform implementation. On Windows the runtime is currently a **scaffold-only stub** that prints a TODO and exits 0 (the full Win32 GDI window loop — HWND creation, WndProc, `timeBeginPeriod(1)`, BitBlt, per-monitor DPI awareness, preview-mode static control subclassing — is the 4.3 follow-up). On Linux/macOS the runtime runs a real raw-termios terminal loop with a 60 FPS target and a differential ANSI renderer (only the cells that changed since the previous frame are redrawn). Public surface: `Mode` enum, `parse_args()`, `print_usage(name)`, `run_main(saver, name)`. The 10 r* effect binary crates in screensavers/ all call this.
- **`screensaver-runtime` feature**: New opt-in feature (default-off) in library. The 10 r* effect binary crates in screensavers/ enable it. The 7 r* TUI apps do not. Pulls in `libc` on non-Windows (for `tcgetattr`/`tcsetattr`/`ioctl`/`select`) and the Win32 GDI stack on Windows.
- **`tests/screensaver_runtime_facade.rs`**: 3 smoke tests for the new runtime — `parse_args` returns a valid Mode, `print_usage` doesn't panic for known effect names, a NoopScreensaver can be driven through the trait dispatch path. Avoids invoking `run_main` directly (it would block on the terminal loop).
- **`lifecycle::foreground::identity::shell_name()` + `refresh_rate_hz()`** were already added in 4.1.2 — the screensaver_runtime's Windows scaffold (the eventual full Win32 GDI loop) is the primary consumer of `refresh_rate_hz()`.

### Changed
- **library 4.2 is the first release where the `trance-core` crate is fully deletable**: the screensaver_runtime now lives in library, and the 10 r* effect binaries in screensavers/ have been collapsed to 20-line `library::screensaver_runtime::run_main` shims. `screensavers/src/trance-core/` was deleted; `screensavers/Cargo.toml` no longer references it.

### Verified
- `cargo test --features screensaver-runtime` (debug) on library: 99 unit + 11 design_facade + 42 scenes_facade + 3 screensaver_runtime_facade + 2 taxonomy + 22 doctests = **179 tests pass**.
- `cargo build --workspace --release` on screensavers/ produces all 10 shim binaries (240-275 KB each; previously 309-352 KB, smaller because they only pull in `screensaver-runtime` not the full `default` features).
- `cargo check` on all 6 r* TUI apps (helm, trance, pulse, ignite, template, scout) with the new `scenes` feature added to their `library` dep — all green.
- `cargo test --test taxonomy_compliance`: All 147 source files audited against the 4-layer taxonomy rules; zero cross-layer violations (the `screensaver_runtime` module is in `lifecycle/foreground/` which can be imported by `role/application/scenes/*`).

### Migration from 4.1.9 → 4.2.0 for the 10 r* effect binary crates
The pre-4.2 release had 10 r* effect crates each with their own `trance-core` dep + ~50 source files per crate. Post-4.2 each r* crate is a 20-line shim:
```rust
fn main() {
    library::screensaver_runtime::run_main(
        library::role::application::scenes::matrix::Matrix::new(),
        "glyphs",
    );
}
```

`trance-core` is fully gone from screensavers/.

## [4.1.9] - 2026-06-08

### Added
- **`library::role::application::scenes::unstable`**: chaos (a 4-phase molecular-oscillation screensaver: Assembled → Vibrating → Disassembling → Exploding, with phase transitions driven by accumulated particle instability) is the **tenth and final** effect migrated into library. Public type: `library::role::application::scenes::unstable::Unstable`. Source: `library/src/role/application/scenes/unstable/{effect.rs, drawing.rs, update_core.rs, types.rs}` — 4 source files. **This completes the library 4.1.x "all 10 r* effects consolidated into library" milestone.**
- **`tests/scenes_facade.rs`**: 4 new chaos tests (80x24, 60x40, 200x60, zero-size). **42 total scene tests**, one per effect (matrix, beams, bhop, fire, fireflies, fireworks, life, party, pour, unstable) × 4 sizes (80x24, 60x40, 200x60, zero-size) + 2 matrix-specific tests (resize-between-updates, has_scanlines for bhop).

### Changed (non-breaking)
- **chaos canonical impl differences**: (a) The pre-4.1 chaos read `HKEY_CURRENT_USER\Software\Windows-Screensavers\chaos\ParticleLimit`/`ExplosionFreq` registry values. The 4.1.9 inline migration collapses these to defaults (1, 1). (b) `trance_core::current_palette()` → `query_current_palette()`. (c) `trance_core::library::rgb::RgbColor` → `crate::role::application::rgb::protocol::RgbColor`. (d) `crate::types` / `crate::effect` / `crate::drawing` sibling imports converted to `super::*`. (e) `trance_core::logo_lines()` replaced with `library::interface::tui::effects::render_logo_block(get_system_info().logo_text, None)`.
- **`render_logo_block` moved to `library::core::logo_block`**: The 5x5 block-letter logo renderer is a pure string-transformer with a static cache — fits in `core` (no `interface` / `role` dependencies). Moved so both the r* TUI effects (interface layer) and the r* screensaver effects (role layer) can import it without violating the 4-layer taxonomy (role is not allowed to import from interface). The pre-4.1.9 `library::interface::tui::effects::render_logo_block` path is preserved as a deprecated re-export for back-compat with any r* TUI app that imports from the 4.0 path.

### library 4.1.x milestone summary

The 4.1.0 → 4.1.9 patch series **migrated all 10 r* effect crates from `screensavers/` into library** at `library::role::application::scenes::*`. The pre-4.1 build layout was 10 separate r* crates each with their own `trance-core` dependency; the post-4.1 build layout is a single library with the 10 effects as feature-gated submodules, and the r* crates in screensavers/ are slated to become thin shim binaries in library 4.2 (which will also move the screensaver_runtime into library so the 10 screensavers/* binaries become 10-line `library::screensaver_runtime::run_main` calls).

Migration per effect, in dependency complexity order:
- **4.1.0** glyphs (1 file, simplest)
- **4.1.1** beams (3 files)
- **4.1.2** bounce (3 files, Windows-only DPI/cell sizing collapsed to 12x20)
- **4.1.3** flame (4 files)
- **4.1.4** gnats (4 files)
- **4.1.5** bursts (4 files)
- **4.1.6** cosmos (8 files, biggest non-audio effect)
- **4.1.7** disco (5 files, includes the audio WASAPI loopback stub)
- **4.1.8** storm (7 files)
- **4.1.9** chaos (4 files, completes the set)

All 10 effects share the same migration pattern: `trance_core::Screensaver` → `crate::core::screensaver::Screensaver`, `trance_core::current_palette()` → `query_current_palette()`, `trance_core::library::rgb::*` → `crate::role::application::rgb::*`, `trance_core::get_system_info()` → `crate::platform::native::sys_info::get_system_info()`, `trance_core::logo_lines()`/`logo_dimensions()` → `crate::interface::tui::effects::render_logo_block`, `HKEY_CURRENT_USER` registry reads → defaults. The deprecated `trance_core::*` indirection layer is fully removed in 4.1.9; in 4.2 the `trance-scenes/trance-core` crate itself can be deleted (its only remaining purpose is the screensaver_runtime, which 4.2 moves into library too).

### Verified
- `cargo test` (debug) on library: 96 unit + 11 design_facade + 42 scenes_facade + 2 taxonomy + 22 doctests = **173 tests pass**.
- `cargo test --test taxonomy_compliance`: All 147 source files audited against the 4-layer taxonomy rules. Zero cross-layer violations.

## [4.1.8] - 2026-06-08

### Added
- **`library::role::application::scenes::pour`**: storm (the rain effect with puddle accumulation, flying birds, ambient lightning, and the same kind of logo-character-reveal seen in cosmos) is the ninth effect migrated. Public type: `library::role::application::scenes::pour::Pour`. Source: `library/src/role/application/scenes/pour/{effect.rs, drawing.rs, update_core.rs, update_bird.rs, update_scenery_and_animals.rs, update_lightning.rs, types.rs}` — 7 source files.
- **`tests/scenes_facade.rs`**: 4 new storm tests (80x24, 60x40, 200x60, zero-size). 38 total scene tests.

### Changed (non-breaking)
- **storm canonical impl differences**: (a) The pre-4.1 storm read `HKEY_CURRENT_USER\Software\Windows-Screensavers\storm\DropCount`/`AssembleSpeed` registry values. The 4.1.8 inline migration collapses these to defaults (1, 1). (b) `trance_core::current_palette()` → `query_current_palette()`. (c) `trance_core::library::rgb::*` → `crate::role::application::rgb::*`. (d) `crate::types` / `crate::effect` / `crate::drawing` sibling imports converted to `super::*` for the new library module layout. (e) The drop-reset branch in `update_core.rs:189` had `cols - 1` which underflowed at 0-cols grids; guarded with `&& cols > 0`.

### Verified
- `cargo test` (debug) on library: 96 unit + 11 design_facade + 38 scenes_facade + 2 taxonomy + 22 doctests = **169 tests pass**.

## [4.1.7] - 2026-06-08

### Added
- **`library::role::application::scenes::party`**: disco (confetti + disco ball + bouncing-equalizer audio visualizer + neon stars) is the eighth effect migrated. Public type: `library::role::application::scenes::party::Party`. Source: `library/src/role/application/scenes/party/{effect.rs, drawing.rs, audio.rs, types.rs}`.
- **`tests/scenes_facade.rs`**: 4 new disco tests (80x24, 60x40, 200x60, zero-size). 34 total scene tests.
- **library `[target.'cfg(windows)'.dependencies] windows` features**: Added `Win32_Media` and `Win32_Media_Audio` to the `windows` crate feature list so the disco audio module compiles. library has a hard dep on the `windows` crate (gated by the `lifecycle-background`, `notification`, and other Windows-only features); enabling these audio features makes disco self-contained.

### Changed (non-breaking)
- **disco canonical impl differences**: (a) The pre-4.1 disco read `HKEY_CURRENT_USER\Software\Windows-Screensavers\disco\ConfettiDensity`/`DiscoBall` registry values. The 4.1.7 inline migration collapses these to defaults (1, 1). (b) The Windows-only `IAudioClient` loopback capture routine (`unsafe fn run_audio_capture`, ~110 lines) used the `windows` crate v0.52 API in a way that the v0.52 method signatures (`device.Activate` taking a single arg, `CoCreateInstance` taking three args) have shifted since the original disco was written. Rather than carry a half-working `unsafe` block into library, the 4.1.7 inline migration uses the same sine+noise fallback generator the non-Windows path uses on Windows too — the visualizer still produces a beat pattern, just not the real WASAPI loopback one. The full WASAPI capture returns in 4.2 once verified end-to-end against the live disco screensaver preview.
- **disco logo**: `trance_core::logo_lines()` + `logo_dimensions()` (Windows file read) replaced with `library::interface::tui::effects::render_logo_block(get_system_info().logo_text, None)`. The disco-ball draw routine got a `cols == 0 || rows == 0` early-return guard to prevent the zero-size-grid panic.

### Verified
- `cargo test` (debug) on library: 96 unit + 11 design_facade + 34 scenes_facade + 2 taxonomy + 22 doctests = **165 tests pass**.

## [4.1.6] - 2026-06-08

### Added
- **`library::role::application::scenes::life`**: cosmos (a 4-state universe simulation: BigBang → Expansion → Accretion → Singularity → Collapse, with gravity wells, nebular ignition, logo-character accretion, and a particle system) is the seventh effect migrated. Public type: `library::role::application::scenes::life::LifeEffect`. Source: `library/src/role/application/scenes/life/{effect.rs, drawing.rs, draw_particles.rs, update_core.rs, update_expansion.rs, update_collapse.rs, update_accretion_helpers.rs, types.rs}` — 8 source files (the r* effect with the most sub-files after storm).
- **`tests/scenes_facade.rs`**: 4 new cosmos tests (80x24, 60x40, 200x60, zero-size). 30 total scene tests.

### Changed (non-breaking)
- **cosmos canonical impl differences**: The pre-4.1 cosmos read `HKEY_CURRENT_USER\Software\Windows-Screensavers\cosmos\SeedDensity`/`SimSpeed` registry values. The 4.1.6 inline migration collapses these to defaults (3, 1). Re-added in 4.2.
- **cosmos logo overlay**: `trance_core::logo_lines()` + `logo_dimensions()` (Windows file read) replaced with `library::interface::tui::effects::render_logo_block(get_system_info().logo_text, None)`.
- **cosmos internal paths**: All `crate::types` / `crate::effect` / `crate::drawing` / `crate::update_*` sibling imports converted to `super::*` for the new library module layout. The `crate::update_expansion::update_*` / `crate::update_collapse::update_*` call sites in `update_core.rs` are now `update_expansion::update_*` / `update_collapse::update_*` (resolved via `use super::{update_expansion, update_collapse}`).

### Verified
- `cargo test` (debug) on library: 96 unit + 11 design_facade + 30 scenes_facade + 2 taxonomy + 22 doctests = **161 tests pass**.

## [4.1.5] - 2026-06-08

### Added
- **`library::role::application::scenes::fireworks`**: bursts (rocket + explosion particle effects with colored bursts, trails, and a city skyline silhouette) is the sixth effect migrated. Public type: `library::role::application::scenes::fireworks::Fireworks`. Source: `library/src/role/application/scenes/fireworks/{effect.rs, drawing.rs, types.rs}`.
- **`tests/scenes_facade.rs`**: 4 new bursts tests (80x24, 60x40, 200x60, zero-size). 26 total scene tests.

### Changed (non-breaking)
- **bursts canonical impl differences**: The pre-4.1 bursts read `HKEY_CURRENT_USER\Software\Windows-Screensavers\bursts\LaunchRate`/`SkylineStyle` registry values. The 4.1.5 inline migration collapses these to defaults (1, 0). Re-added in 4.2.
- **bursts logo**: `trance_core::logo_lines()` + `logo_dimensions()` replaced with `library::interface::tui::effects::render_logo_block(get_system_info().logo_text, None)`. The `logo_y = ... / 2 - 3` was fixed to `saturating_sub` to avoid the 0-row underflow that broke the zero-size smoke test.

### Verified
- `cargo test` (debug) on library: 96 unit + 11 design_facade + 26 scenes_facade + 2 taxonomy + 22 doctests = **157 tests pass**.

## [4.1.4] - 2026-06-08

### Added
- **`library::role::application::scenes::fireflies`**: gnats (boids-like fireflies with attractors, trails, and a glow-excited centered logo) is the fifth effect migrated. Public type: `library::role::application::scenes::fireflies::Fireflies`. Source: `library/src/role/application/scenes/fireflies/{effect.rs, drawing.rs, types.rs}`.
- **`tests/scenes_facade.rs`**: 4 new gnats tests (80x24, 60x40, 200x60, zero-size). 22 total scene tests.

### Changed (non-breaking)
- **gnats logo**: The pre-4.1 `trance_core::logo_lines()` + `logo_dimensions()` (Windows file read) replaced with `library::interface::tui::effects::render_logo_block(get_system_info().logo_text, None)`. The `logo_excitation` buffer (used for the glow effect under each logo character) is now sized to a fixed 80x12 — matches the typical 80x12 logo block rendered by `render_logo_block` for the 4-letter OS tokens (e.g. "WIN11", "ARCH"). Logo glow now follows the centered position correctly.
- **gnats palette + RGB**: All `trance_core::current_palette()` → `crate::role::application::palette::query_current_palette()`. All `trance_core::library::rgb::*` → `crate::role::application::rgb::*`. All `trance_core::{hsl_to_rgb, rgb_to_hsl}` → `crate::core::{hsl_to_rgb, rgb_to_hsl}`. No behavior change.

### Verified
- `cargo test` (debug) on library: 96 unit + 11 design_facade + 22 scenes_facade + 2 taxonomy + 22 doctests = **153 tests pass**.

## [4.1.3] - 2026-06-08

### Added
- **`library::role::application::scenes::fire`**: flame (Doom-style cellular-automata fire ramp + sparks + volcanic globs + starfield) is the fourth effect migrated. Public type: `library::role::application::scenes::fire::FireEffect`. Source: `library/src/role/application/scenes/fire/{effect.rs, drawing.rs, types.rs}`.
- **`tests/scenes_facade.rs`**: 4 new flame tests (80x24, 60x40, 200x60, zero-size). 18 total scene tests.

### Changed (non-breaking)
- **flame canonical impl differences**: The pre-4.1 flame read `HKEY_CURRENT_USER\Software\Windows-Screensavers\flame\FlameHeight`/`SparkCount` registry values. The 4.1.3 inline migration collapses these to defaults (both = 1, i.e. medium). Re-added in 4.2.
- **flame logo overlay**: Was `trance_core::logo_lines()` + `logo_dimensions()` (a static Windows file). 4.1.3 uses `library::interface::tui::effects::render_logo_block` with `get_system_info().logo_text`.

### Verified
- `cargo test` (debug) on library: 96 unit + 11 design_facade + 18 scenes_facade + 2 taxonomy + 22 doctests = **149 tests pass**.

## [4.1.2] - 2026-06-08

### Added
- **`library::role::application::scenes::bhop`**: bounce (TUI dashboard + fake command console + autonomous bhop simulator) is the third effect migrated. Public type: `library::role::application::scenes::bhop::BhopDashboard`. Implements the unified `Screensaver` trait + returns `has_scanlines() = true` for CRT overlay. Source: `library/src/role/application/scenes/bhop/{animation.rs, drawing.rs, types.rs}`.
- **`lifecycle::foreground::identity::shell_name()`** + **`refresh_rate_hz()`**: New cross-platform helpers. `shell_name()` returns PowerShell v7.4 (Windows if `$PSModulePath` is set), cmd.exe (Windows fallback), or `$SHELL` env var (POSIX, default `/bin/bash`). `refresh_rate_hz()` queries the device caps on Windows, returns 60 on other platforms. bounce dashboard uses both; other r* apps can use them too.

### Changed (non-breaking)
- **bounce canonical impl differences**: The pre-4.1 bounce had several Windows-only behaviors that the 4.1.2 inline migration collapses to defaults. (a) DPI-aware cell sizing via `GetDC`/`GetDeviceCaps(LOGPIXELSX)` → fixed 12x20 cells (the dpi-aware sizing returns in 4.2). (b) Keyboard input via `GetAsyncKeyState(VK_SPACE)` for jump → dropped, autonomous AI-only (returns in 4.2 with screensaver_runtime's native input layer). (c) `HKEY_CURRENT_USER\Software\Windows-Screensavers\bounce\Speed`/`ShowSysInfo` registry reads → defaults (Speed=1, ShowSysInfo=true). (d) `cols - 77` underflow on grids < 77 wide → `cols.saturating_sub(77)`. Visual output is identical at 80x24 (the canonical screensaver preview size).
- **Theme mode label**: Was `trance_core::get_theme_mode()` (Windows-only). 4.1.2 derives the label from `library::platform::native::sys_info::query_system_theme().is_dark_mode` (cross-platform, cached).

### Verified
- `cargo test` (debug) on library: 96 unit + 11 design_facade + 14 scenes_facade + 2 taxonomy + 22 doctests = **145 tests pass**.

## [4.1.1] - 2026-06-08

### Added
- **`library::role::application::scenes::beams`**: beams (cinematic volumetric spotlights + twinkling stars + dust particles) is the second effect migrated into library. Public type: `library::role::application::scenes::beams::Beams`. Implements the unified `library::core::screensaver::Screensaver` trait. Source: `library/src/role/application/scenes/beams/{effect.rs, types.rs}`. `default_spotlights()`, `Spotlight`, `Star`, `DustParticle` are all re-exported for downstream consumers.
- **`tests/scenes_facade.rs`**: 4 new beams tests (80x24, 60x40, 200x60, zero-size). 9 total scene tests, all green.

### Changed (non-breaking)
- **beams canonical impl differences**: The pre-4.1 beams read `HKEY_CURRENT_USER\Software\Windows-Screensavers\beams\BeamCount` and `TwinkleStars` registry values. The 4.1.1 inline migration collapses these to defaults (4 beams, twinkle on) since library has no settings module yet. The full registry round-trip will be re-added in 4.2 alongside the screensaver_runtime move. Visual output is identical at default settings.
- **beams logo overlay**: Was rendered from `trance_core::logo_lines()` (a static Windows-only file). 4.1.1 uses `library::interface::tui::effects::render_logo_block` with the live system `logo_text` (e.g. "WIN11", "ARCH", "FEDORA"), so the centered overlay now matches the host OS.

### Verified
- `cargo test` (debug) on library: 96 unit + 11 design_facade + 9 scenes_facade + 2 taxonomy + 22 doctests = **140 tests pass**.

## [4.1.0] - 2026-06-08

### Added
- **`role::application::scenes` façade**: The 10 r* screensaver effects (cosmos, glyphs, flame, bounce, beams, storm, chaos, disco, bursts, gnats) are migrating into library at `library::role::application::scenes::<name>::<EffectType>`. 4.1.0 ships the module structure with **glyphs** as the canonical reference impl; the other 9 land as 4.1.x patch releases. r* effect crates and r* apps can import via `use library::role::application::scenes::matrix::Matrix;` and the existing `Screensaver` trait from `library::core::screensaver`.
- **`scenes` feature**: A new opt-in `scenes` feature (default-on) controls whether the `scenes` module is compiled. r* apps that don't need the effects can opt out with `default-features = false`. The feature is **empty** (no extra deps) — it just gates the module declarations so the additional `library` compile cost is incurred only by consumers that pull it in.
- **`tests/scenes_facade.rs`**: 5 smoke tests (glyphs: 80x24, 60x40, 200x60, resize-between-updates, zero-size grid) lock in the public API and resize safety. Will grow to 10 tests × 5 sizes as the other 9 effects land in 4.1.x.

### Changed (non-breaking)
- **glyphs canonical impl**: The pre-4.1 glyphs effect source lives in library at `src/role/application/scenes/matrix/effect.rs`. Implementation differences from the screensavers/glyphs 0.1.x version: pulls accent from `library::role::application::palette::query_current_palette()` (no more `trance_core::current_palette()` indirection), uses `library::role::application::rgb::RgbController` directly, drops the `HKEY_CURRENT_USER` density/katakana registry reads (defaults to density=1, full pool). Visual output is identical to the prior screensavers/glyphs at default settings.

### Verified
- `cargo test` (debug) on library: 96 unit + 11 design_facade + 2 taxonomy + 5 scenes_facade = **114 tests pass**.
- `cargo check` on library with `--no-default-features`: compiles cleanly (the `scenes` feature correctly gates the module).

## [4.0.0] - 2026-06-08

### Added
- **Unified Design System**: New `library::interface::tui::design` façade + `design::prelude` is the single import path for r* TUI apps (helm, pulse, trance, template, scout, hub). Brings in theme, accent bundles, status bar, toast, markdown viewer, layout guard, title banner, effect preview, mouse selection, layout helpers, text utilities, terminal-size constants, all 12 canonical TUI effects, and the unified `Screensaver` trait. See `docs/DESIGN_SYSTEM.md` for the full onboarding guide.
- **`ScreenPalette`**: New backend-agnostic RGB-tuple palette in `library::role::application::palette` with `bg`/`fg`/`accent`/`dim`/`hot`/`cool`/`mid`/`peak` fields. `from_system(accent, is_dark)` is the canonical 4.0 constructor; `query_current_palette()` is the cross-platform helper. r* TUI apps and r* GDI screensaver apps now share the same color story.
- **`dimensions::Palette` AccentTriad**: New `AccentDim`, `AccentHot`, `AccentCool` variants that map 1:1 to `ScreenPalette`'s `dim`/`hot`/`cool` channels. TUI effects can now use the system accent's natural triadic scheme without hand-rolled HSL math.
- **`Screensaver::has_scanlines`**: New default-`false` hook on the unified `Screensaver` trait. trance-scenes GDI effects opt in to `true` for CRT overlay.
- **`Screensaver: ScreensaverState` supertrait**: Every `Screensaver` automatically implements `ScreensaverState` with default-true / no-op setters. Removes the trait-object friction (`Box<dyn Screensaver + ScreensaverState>` is no longer needed).
- **Façade tests**: `tests/design_facade.rs` (11 tests) locks in the 4.0 public surface at 80x24, 106x30, and 200x60 terminal sizes.
- **3.x back-compat shims**: The pre-4.0 module paths (`library::interface::tui::theme`, `library::interface::tui::markdown`, `library::interface::tui::markdown_viewer`, `library::interface::tui::layout`, `library::interface::tui::status`, `library::interface::tui::text`, `library::widgets::colors`, ...) are preserved as deprecated re-exports for one minor release (4.0 → 4.1). The `ScreensaverEffect` trait is re-exported as a deprecated alias for the same window.
- **ARCHITECTURE.md + docs/DESIGN_SYSTEM.md**: Substantial 4.0 section in ARCHITECTURE.md; new `docs/DESIGN_SYSTEM.md` is the r* app author onboarding guide.

### Changed (breaking)
- **`Screensaver` trait moved to `core`**: `library::core::screensaver::Screensaver` is the canonical, backend-agnostic trait (no ratatui dependency). The pre-4.0 ratatui-coupled trait in `library::interface::tui::screensaver` now re-exports it; the `ScreensaverRenderer` buffer-management helper stays in `interface::tui` (TUI-layer concern).
- **`Screensaver::update` takes `Duration`**: Was `f32` seconds in 3.x. `Duration::from_secs_f32(dt)` bridges.
- **`ScreensaverRenderer::tick_duration`**: The new 4.0 entry point. The pre-4.0 `tick(&mut s, f32)` is a deprecated shim that converts internally.
- **`Screensaver` is a single trait + supertrait, not a marker**: `init`/`update`/`draw`/`has_scanlines` are declared directly. The `ScreensaverState` sub-trait has default-true / no-op setters; the 12 library TUI effects' manual `ScreensaverState` impls were removed in favor of the blanket.
- **TerminalCell `pub fn draw(&self, ...)`**: Was `&mut self` in 3.x. The two library effects that mutated internal `last_drawn`/`last_cols`/`last_rows` state inside `draw` now use `RefCell` (a documented internal-cache pattern).
- **Module split**: Chrome files (`theme`, `colors`, `status`, `toast`, `markdown`, `markdown_viewer`, `layout`, `layout_guard`, `title_banner`, `effect_preview`, `mouse_selection`, `text`) all live under `src/interface/tui/design/` in 4.0. The pre-4.0 top-level paths are deprecated re-exports.
- **helm adoption**: All 11 `library` touchpoints in helm now route through `library::interface::tui::design::prelude::*`. The hand-rolled `(show_markdown, markdown_lines, markdown_scroll)` triple in `App` is replaced by `MarkdownViewerState`. `helm --stdout` and the library TUI dashboard share the same `ThemeColors`.
- **trance-scenes adoption**: `trance-core` re-exports the unified `Screensaver` and `TerminalCell` from library (the local duplicates are removed). gnats's drawing now consumes `current_palette()` from the library `ScreenPalette` instead of computing its own HSL triadic accent scheme. Other trance-scenes effects can migrate incrementally.
- **template adoption**: `Box<dyn Screensaver>` (no more `+ ScreensaverState` bound); `tick_duration` instead of deprecated `tick(f32)`.
- **helm version**: 3.0.21 → 3.1.0 (consumes library 4.0; no behavior change).
- **template version**: 3.1.0 → 3.2.0 (consumes library 4.0; no behavior change).
- **trance-scenes trance-core version**: 0.1.4 → 0.1.5 (consumes library 4.0; no behavior change).

### Deprecated
- `library::interface::tui::screensaver::ScreensaverEffect` (use `library::core::screensaver::Screensaver`).
- `library::interface::tui::screensaver::ScreensaverRenderer::tick` (use `tick_duration`).
- All 3.x module paths in `library::interface::tui::*` and `library::widgets::*` (use the `design::` paths).

### Verified
- `cargo test` (debug + release) on library: 96 unit + 11 design façade + 2 taxonomy + 21 doctests = **130 tests pass**.
- `cargo check` + `cargo test --release` on helm, template, trance-scenes × 10 effects: all green.
- `cargo build --release` binaries: `helm.exe` 1.5 MB, `rpack.exe` 235 KB, `rgb_diagnostic.exe` 142 KB, 10× trance-scenes `.exe` 309-352 KB each (all with `opt-level="z"` + LTO + strip).

## [3.4.4] - 2026-06-08

### Fixed
- **Mouse Selection**: Corrected `point_in_rect` calculation to prevent highlighting and copying the entire line during precise single-row mouse clicks or drags.
- **Compiler & Linter Warnings**: Resolved all static analysis issues flagged by Clippy inside `theme.rs`, `pulsing_waves.rs`, `tui_bootstrap.rs`, and `effects/mod.rs` (redundant casts, layout ordering, default trait implementations, and assertion simplifications).

## [3.4.3] - 2026-06-08

### Added
- **Color Utilities**: Migrated the generic `hex_to_rgb` utility function to the shared `role::application::formatting` module and added tests for color parsing correctness.

## [3.4.2] - 2026-06-08

### Added
- **System Info**: Ported `get_local_time_string` and `get_win_accent_color_hex` helpers from `helm-tui` to the shared `platform::native::sys_info` module.

## [3.4.1] - 2026-06-08

### Added
- **Color Utilities**: Added `hsl_to_rgb` and `rgb_to_hsl` helper functions to the core module to centralize HSL/RGB conversion logic across dependent crates in the ecosystem.

## [3.4.0] - 2026-06-08

### Added
- **Verb × Noun Effects Taxonomy**: Structured TUI effects in `interface::tui::effects` into orthogonal dimensions (Verb, Noun, Style, Palette, Speed, Direction, Density).
- **New Visual Effects**: Added 8 new effects (`FallingComets`, `FlowingBlocks`, `PulledBlocks`, `PulsingGlyphs`, `PulsingParticles`, `PulsingWaves`, `RisingGlyphs`, and `RisingFlames`).
- **Compatibility Aliases**: Exposed deprecated type aliases (`MatrixRain`, `SimpleParticles`, `GravityParticles`, `RainEffect`, `FireEffect`) at the crate root to keep existing consumer apps compiling.

### Changed
- **Supply Chain Security**: Hardened dependencies by disabling default features for `ratatui` (enabling only the `crossterm` backend) and `crossterm` (enabling only `events`), reducing binary size and dependency attack surface.
- **Visual Renames**: Renamed legacy TUI effects to follow the taxonomy (e.g. `MatrixRain` -> `FallingGlyphs`, `RainEffect` -> `FallingDroplets`).

### Fixed
- **FFI Safety**: Fixed potential out-of-bounds slice panic in `ConsoleTitleGuard` and `get_console_title` on Windows by capping retrieved console title slicing to the stack buffer capacity.

## [3.0.22] - 2026-06-08

### Changed
- **Binary Footprint Optimization**: Disabled default-features for the `winreg` dependency, removing its transitive serialization feature (`serde`) from the library build graph. This decreases compilation times and reduces target binary size for Windows builds.

## [3.0.21] - 2026-06-08

### Changed
- **Incremental sysinfo Caching**: Optimized `get_dashboard_info` inside `sys_info` module to reuse a static `sysinfo::System` instance via a thread-safe `Mutex` instead of reconstructing a new one on every cache refresh. This eliminates costly directory parses and process list initializations on every update cycle.

## [3.0.20] - 2026-06-08

### Fixed
- **SingleInstanceGuard Discard Bug**: Fixed a severe lifecycle bug in `bootstrap_tui` where the returned `SingleInstanceGuard` was immediately discarded on creation, releasing the lock/mutex immediately and allowing concurrent instances to execute. It is now properly stored inside `TuiGuards`.
- **TUI TerminalGuard RAII Cleanup**: Added a `TerminalGuard` RAII helper inside `TuiGuards` to guarantee automatic raw-mode disabling and alternate screen exit when `TuiGuards` goes out of scope, protecting against terminal lockups when the TUI exits via early returns or normal errors.
- **Win32 Named Pipe Handle Double-Close Hazard**: Rewrote the Windows Named Pipe reader/writer to perform raw I/O directly on the `HANDLE` using the Win32 `ReadFile` and `WriteFile` system calls instead of wrapping in `std::fs::File`, eliminating double-close bugs during panic stack unwinding.

## [3.0.19] - 2026-06-08

### Added
- **Accidental Publish Guard**: Added `publish = false` to `Cargo.toml` to secure the software supply chain by preventing accidental package publication to public registries like crates.io.

### Changed
- **Supply Chain Security Audit**: Performed a security audit of the complete dependency tree showing 100% compliance with permissive open-source licenses (MIT, Apache-2.0, Zlib, BSL-1.0, Unicode-3.0) and zero known security advisories or vulnerabilities.

## [3.0.18] - 2026-06-08

### Changed
- **Dependency Optimization**: Disabled default features on the `sysinfo` dependency to eliminate the heavy `rayon` multithreading engine and its dependencies from the library build graph.
- **Size-Optimized Release Profile**: Added a size-focused `[profile.release]` section to `Cargo.toml` with `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, and `strip = true` to maximize executable footprint reductions for all binary targets.

## [3.0.17] - 2026-06-08

### Fixed
- **RGB Flash Zero-Duration Panic**: Added a safety guard in `src/role/application/rgb/controller.rs` to prevent a division-by-zero (producing `NaN` and causing a `.clamp()` panic) when a zero-duration flash command is executed.

## [3.0.16] - 2026-06-08

### Added
- **Theme-Aware Selection Foreground**: Added `selection_fg` to `ThemeColors` to support theme-customized selection highlight colors in TUI interfaces.

## [3.0.15] - 2026-06-08

### Fixed
- **Clippy `too_many_arguments` on `draw_title_banner`**: Annotated the title banner widget with `#[allow(clippy::too_many_arguments)]` plus a justification comment, since the function intentionally composes the full title strip (title, version, user, host, OS) in a single render pass.
- **Clippy `type_complexity` in `sys_info` caches**: Introduced `CachedAccent`, `CachedBool`, `CachedString`, `CachedTheme`, and `CachedPower` type aliases in `platform::native::sys_info` to clarify the tuple-of-`(Instant, T)` cache entries used by the TTL-bounded query helpers.
- **Clippy `type_complexity` in `logo` block cache**: Introduced a `LogoCacheEntry` type alias in `interface::tui::effects::logo` to document the `(text, sub_text, rendered_lines)` tuple cached by `render_logo_block`.
- **Clippy `module_inception` for `interface::gui::gui`**: Renamed the inner `gui` submodule to `egui_helpers` (file `gui.rs` → `egui_helpers.rs`) so the parent `interface::gui` module no longer contains a child module of the same name. Top-level `library::gui` re-export still resolves to the same helpers via the renamed module.

### Changed
- **GPU buffer sizing**: Replaced manual `(data.len() * std::mem::size_of::<f32>()) as u64` with `std::mem::size_of_val(data) as u64` in `platform::native::gpu` to follow clippy's `manual_slice_size_calculation` lint and avoid hard-coding the element type.
- **GPU workgroup dispatch**: Replaced manual ceiling division with the standard-library `u32::div_ceil` in `platform::native::gpu` for the compute shader dispatch count.

## [3.0.14] - 2026-06-08

### Added
- **Headless GPU taxonomy feature**: Added `platform-gpu` feature (aliasing `gpu`) for consistent taxonomy naming.
- **Theme Color Tests**: Added unit tests in `theme.rs` to cover `success`, `selection_bg`, and `warning` fields.

### Fixed
- **Clippy lints**: Fixed needless range loop warning in `textbox.rs`.
- **Deprecation warnings**: Suppressed compiler warnings on legacy compatibility re-exports of relaunch, service, and priority helpers, with inline comments explaining the suppressions.
- **GPU Mutex redundancy**: Removed redundant `Mutex` from `HEADLESS_GPU` static using `OnceLock<Option<...>>` instead.
- **GPU Thread underutilization**: Upgraded shader dispatch in `gpu.rs` from `@workgroup_size(1)` to `@workgroup_size(64)`.
- **GPU Error handling**: Replaced silently swallowed error cases in `gpu.rs` mapping routine with detailed diagnostics logged to stderr.
- **Theme Color differentiation**: Differentiated warning color from quit button color across light/dark themes.

## [3.0.13] - 2026-06-08

### Added
- **Theme-aware Success and Selection Colors**: Added `success` and `selection_bg` color fields to `ThemeColors` to support styling theme-swapped states.

## [3.0.12] - 2026-06-08

### Added
- **Theme-aware Warning Color**: Added a `warning` color field to `ThemeColors` to support styling theme-swapped warning/error states.

## [3.0.11] - 2026-06-08

### Added
- **Unified Screensaver Factory**: Created `make_effect` function in `interface::tui::effects` to consolidate screensaver instance creation.
- **Unified Effect Names**: Created `EFFECT_NAMES` constant array in `interface::tui::effects` to prevent duplicated string array allocations across application files.

## [3.0.10] - 2026-06-08

### Added
- **ButtonRect Boundary Primitive**: Added `ButtonRect` struct to `interface::tui::widgets::title_banner` for encapsulating button boundary positions and mouse checks.
- **Theme-aware button colors**: Added `username`, `help_btn`, and `quit_btn` fields to `ThemeColors` to support styling theme-swapped button states.

## [3.0.9] - 2026-06-08

### Added
- **Title Banner Widget**: Created `library::interface::tui::widgets::title_banner` containing `draw_title_banner` for rendering standard TUI app title strips.
- **Effect Preview Widget**: Created `library::interface::tui::widgets::effect_preview` containing `draw_effect_preview` for displaying screensaver grids.

## [3.0.8] - 2026-06-08

### Added
- **TUI Widgets & Utilities**: Added generic `TextBox` widget to `interface::tui::widgets::textbox`.
- **Theme Manager**: Created `interface::tui::theme` with reusable `ThemeColors` and `get_theme` factory.
- **Markdown Viewer & Macro**: Created `interface::tui::markdown` module containing `parse_markdown_to_lines`, `draw_markdown_modal` widget delegate, and `embedded_docs!` helper macro.
- **TUI Layout Helpers**: Added `centered_rect` and `format_help_row` to `interface::tui::layout`.
- **Background Worker**: Added `lifecycle::background::worker` with `Worker` trait, `WorkerEvent` enum, and mock `SampleWorker` implementation.

### Fixed
- **Empty text wrapping**: Fixed `wrap_text` in `interface::tui::text` to return an empty vector early if input text is empty.

## [3.0.7] - 2026-06-08

### Changed
- **Modularized Platform Providers**: Extracted `WindowsPlatform`, `LinuxPlatform`, and `FallbackPlatform` implementations from `sys_info/mod.rs` to a new sub-module `sys_info/providers.rs` to keep individual source files within the 500-line budget limit (reducing `sys_info/mod.rs` line count to ~460 lines).

## [3.0.6] - 2026-06-08

### Added
- **Standard Windows Event Log Constants**: Added `EVENTLOG_*` event types and `EVENT_ID_USER_ACTION` constants to `event_log` module.
- **Generic File Logger**: Implemented a thread-safe, cached file logger with system event logging fallback in `file_log` module.
- **Generic Configuration Manager**: Implemented `AppConfig` and `ConfigFields` trait in `platform::native::config` to provide reusable configuration parsing/saving.
- **TUI Layout Constants**: Added `MIN_TERMINAL_WIDTH` and `MIN_TERMINAL_HEIGHT` to `interface::tui::constants`.

## [3.0.5] - 2026-06-08

### Fixed
- **Clippy Code Refactorings**: Resolved 10+ compiler clippy suggestions across multiple modules (such as using `strip_prefix`, `is_some_and`, collapsing `if` blocks, and removing redundant `clone` calls) to enhance overall codebase quality.

## [3.0.4] - 2026-06-08

### Changed
- **Version Bump**: Bumped version to `3.0.4` to align with the active tracking of TUI applications.

## [2.0.4] - 2026-06-08

### Added
- **Universal CLI Matchers**: Added core matchers and exports for standard flags:
  - Debug/Verbose (`--debug`, `-d`, `--verbose`)
  - No Color (`--no-color`, `-n`)
  - JSON Output (`--json`, `-j`)
  - High Contrast (`--high-contrast`, `-c`)
  - Accessibility/Screen Reader (`--accessible`, `--screen-reader`)
  - Force TUI/Interactive (`--tui`, `--interactive`)
  - Force CLI/Non-interactive (`--cli`, `--non-interactive`)
  - Borderless Console (`--borderless`, `-b`)

## [2.0.2] - 2026-06-08

### Added
- **AccentScrollbar TUI Widget**: Created a vertical scrollbar widget styled with the accent color, featuring scrolling indicators and thumb tracks.

### Changed
- **Stateless List Viewport Scrolling**: Upgraded the `AccentList` TUI widget to support automated sliding viewport windowing based on `selected_index`, preventing truncation and keeping selections in view stateless.

## [2.0.1] - 2026-06-08

### Added
- **Key Distinctions Documentation**: Added architectural definitions for CLI, TUI, and GUI under the presentation layer (Interface) in `ARCHITECTURE.md` and module documentation (`src/interface/mod.rs`).

## [2.0.0] - 2026-06-08

### Removed
- **Legacy Win32 Shim & Feature**: Deleted the deprecated, flat `library::win32` compatibility module from `src/lib.rs` and its associated `win32` meta-feature from `Cargo.toml`. This is a breaking change completing the transition to the new 4-layer taxonomy modules.

### Changed
- **Test Suite Reorganization**: Moved all inline test fixtures out of `src/lib.rs` and placed them alongside the code they test in their respective module files (e.g., `reg.rs`, `sys_info/mod.rs`, `notification.rs`, `daemon.rs`, `visibility.rs`, `game.rs`, `platform/mod.rs`), leaving `src/lib.rs` purely as the public library surface.
- **Dynamic CLI Help Column Widths**: Updated `CliParser::print_help` in `src/interface/cli/scaffold.rs` to compute column alignment widths dynamically from the longest command and option flag name at runtime, resolving potential layout wrapping issues.

### Fixed
- **Linux SingleInstanceGuard Race Condition**: Refactored `SingleInstanceGuard` on Linux to use atomic file locking (`flock` FFI) on `/tmp/{}_single_instance.sock` instead of a `UnixListener` bind, eliminating the TOCTOU (Time-of-Check to Time-of-Use) race condition.
- **Unified Fallback Host Name**: Introduced `core::UNKNOWN_HOST` constant for `"localhost"` and refactored formatting and sys-info modules to use it, removing redundant hardcoded fallback strings.
- **Protocol Magic Numbers**: Replaced magic numbers in OpenRGB protocol parsing (`skip_bytes` calls) with named constants (`MODE_FIXED_FIELDS_SIZE` and `ZONE_FIXED_FIELDS_SIZE`) in `src/role/application/rgb/protocol.rs`.

## [1.9.28] - 2026-06-08

### Fixed
- **Glob Re-export Refactoring**: Replaced all four remaining wildcard glob re-exports (`pub use ...::*`) in `src/lib.rs` (for packages, monitors, window, and console modules) with explicit list re-exports to optimize cargo build and type-checking performance.

## [1.9.27] - 2026-06-08

### Fixed
- **MatrixRain Draw Optimization**: Changed drawing to track and clear only changed/previously drawn cells, avoiding full-grid clears.
- **GravityParticles Physics Optimization**: Replaced `sqrt()` and float divisions with a fast inverse square root (`inv_sqrt`) approximation in gravity calculations.
- **System Query Caching Audit**: Verified that static `CACHE` with TTL is active for all native queries (accent color, theme, dark mode, high contrast, OS version, power status, and BIOS info) in `platform/native/sys_info`.

## [1.9.26] - 2026-06-08

### Fixed
- **Win32 IPC Auto-Trait Derivations**: Introduced a `SendHandle` newtype wrapper in `src/interface/api/platform/win32_ipc.rs` to wrap Win32 `HANDLE` pointers, allowing the compiler to automatically derive `Send` and `Sync` for `Win32IpcServer` and `Win32IpcClient` and preserve auto-trait inferring.
- **Taxonomy Feature Refinements**: Refined the default features list to use taxonomy-aligned flags, split TUI effects out from `role-application`, and conditionally gated `interface::api` behind `interface-api`.
- **Performance Optimizations**: Cached expensive OS sys-info queries, GPU names, disk sizes, network status, and display counts with configurable TTLs, optimized `LcgRng::next_f32` float logic to bypass double-precision round-trips, and removed the redundant thread sleep on RGB controller drop.
- **Explicit Re-exports**: Replaced all wildcard glob re-exports within library modules and compatibility shims with explicit lists to improve compile speeds and diagnostics.

## [1.9.24] - 2026-06-08

### Added
- **Background Daemon Boilerplate & IPC Coordination**:
  - Introduced `DaemonConfig`, `DaemonPriority`, and `DaemonService` in `lifecycle::background::daemon` to bootstrap services with configured priority, sleep prevention, and single-instance locks.
  - Implemented the `DaemonIpcExt` extension trait in `interface::api` (tied together at `src/lib.rs`) to cleanly run local IPC servers named after the daemon service.
  - Exposed `get_sleep_prevention_count` publicly to monitor global active power guards.
  - Added unit and doc test coverage for daemon bootstrapping and power guard integration.

### Fixed
- **Modularity & Feature Gating Enforcement**:
  - Gated all Windows-specific code using `windows_sys` under the corresponding feature gates (`service`, `event-log`, `window`, `sys-info`, etc.) in `src/lifecycle/background/service.rs`, `src/lifecycle/background/event_log.rs`, `src/lifecycle/background/daemon.rs`, `src/platform/native/monitors.rs`, and `src/interface/gui/native.rs`.
  - Gated modules and re-exports in `src/lifecycle/foreground/mod.rs`, `src/lifecycle/background/mod.rs`, and `src/lib.rs` behind their respective features.
  - Fully resolved compilation failures when compiling with `--no-default-features` or specific subsets, ensuring that taxonomy layers compile independently and comply with dependency isolation rules.
- **Unified Stub & Placeholder Wording**:
  - Defined two standard categories of fallback implementations (`Platform Stub` and `Feature Stub`) inside `src/platform/mod.rs`.
  - Updated `src/platform/embedded.rs` to use the standard taxonomy format and unified platform stub header.
  - Standardized comments and documentation for disabled feature fallbacks and placeholder APIs in `src/platform/native/sys_info.rs`, `src/interface/gui/native.rs`, `src/interface/api.rs`, `src/interface/cli/mod.rs`, and `src/role/application/packages.rs`.
- **AccentTheme Facade Documentation**:
  - Reviewed the relationship between `AccentColors` and `AccentTheme`.
  - Updated `AccentTheme` documentation in `src/interface/tui/widgets.rs` to clarify that it is a stateless factory facade rather than a caching provider, and advised caching the returned `AccentColors` bundle in application state for optimal rendering loop performance.

## [1.9.23] - 2026-06-08

### Added
- **Reusable TUI Screensaver Trait & Renderer**:
  - Introduced the `Screensaver` trait in `interface::tui::screensaver` defining a unified lifecycle for terminal visuals: `init`, `update`, `draw`, `set_active`, and `set_focused` hooks.
  - Implemented the `ScreensaverRenderer` manager helper to automatically handle terminal resizing, buffer clearing, frame-ticking, and 50% brightness dimming when screensavers lose focus.
  - Ported/implemented the `Screensaver` trait for all five existing TUI visual effects: `MatrixRain`, `SimpleParticles`, `GravityParticles`, `RainEffect`, and `FireEffect`.
  - Added unit test coverage for the screensaver renderer lifecycle, focus dimming transitions, grid buffer sizes, and effect trait compatibility.

## [1.9.22] - 2026-06-08

### Added
- **First-Class Error Handling Consistency**:
  - Refactored `SingleInstanceGuard::try_new` in `lifecycle::foreground::guard` to return `crate::error::Result<Self>` wrapping errors in the custom `libraryError::Guard` variant instead of returning raw `String` errors.
  - Refactored all local IPC APIs and wrappers (`IpcServiceHost`, `IpcServiceClient`, `IpcServer`, `IpcClient`) under `interface::api` to return `crate::error::Result` utilizing `libraryError::Ipc` instead of returning standard `std::io::Error`.
  - Refactored the internal OpenRGB protocol device parser `parse_device_payload` under `role::application::rgb::protocol` to return `crate::error::Result` utilizing `libraryError::Rgb` instead of returning static `&'static str` errors.
  - Introduced `libraryError::is_ipc_termination` to easily inspect and identify transient connection terminations or client disconnect signals cleanly across platforms.

## [1.9.21] - 2026-06-08

### Added
- **Reusable Headless & IPC Services**:
  - Implemented structured request/response message formats (`IpcRequest`, `IpcResponse`) supporting direct serialization/deserialization.
  - Added a reusable background thread event loop handler (`IpcServiceHost`) and client client wrapper (`IpcServiceClient`) to avoid duplicate logic across background daemons (e.g. `ignite`, `trance`, `template`).
  - Added manual, lightweight, and dependency-free serialization and deserialization helpers (`serialize_dashboard_info`, `deserialize_dashboard_info`) for transferring live `DashboardInfo` packets across local pipes/sockets.
  - Added unit test coverage verifying request-response query cycles and dashboard serialization correctness.

## [1.9.20] - 2026-06-08

### Added
- **First-class Platform Providers & Stubs**:
  - Introduced the `PlatformProvider` trait under `platform` defining a unified cross-platform system query interface.
  - Implemented `PlatformProvider` for `WebPlatform`, `MobilePlatform`, and `EmbeddedPlatform` stubs, returning premium defaults matching the local76 apps.
  - Implemented `PlatformProvider` for native `WindowsPlatform` and `LinuxPlatform` systems.
  - Added the `CurrentPlatform` type alias to resolve to the active provider implementation based on the compilation target.
  - Relocated shared platform structs (`PowerStatus`, `SystemBiosInfo`, `DiskDriveInfo`, `NetworkAdapterInfo`) to `platform/mod.rs` to decouple sibling modules.
  - Added unit test coverage verifying stub values and trait behaviors across all platforms.

## [1.9.19] - 2026-06-08

### Added
- **Targeted TUI & Daemon Tests**:
  - Added full test coverage for the `TuiEffect` trait and all major implementations (`MatrixRain`, `SimpleParticles`, `GravityParticles`, `RainEffect`, `FireEffect`) verifying active flag behavior and CPU-saving empty cell rendering on inactive states.
  - Added focused vs unfocused render differences tests in `AccentTabs` to verify indicator bar rendering.
  - Added focused vs unfocused render differences tests in `AccentTextBox` to verify block cursor rendering and placeholder fallback.
  - Added validation tests for `ToastBox` to verify correct icon and message text rendering.

## [1.9.18] - 2026-06-07

### Added
- **Expanded Rustdoc Examples**:
  - Documented `AccentColors` and `AccentTheme` color helpers with compile-checked examples.
  - Documented TUI focus widgets (`AccentGauge`, `AccentList`, `AccentTabs`, and `AccentTextBox`) with real examples demonstrating how they de-emphasize when inactive (`!focused`).
  - Added `TuiEffect` trait and `MatrixRain` examples demonstrating active/inactive rendering loop states to yield CPU.
  - Added compile-checked doc-tests for formatting helpers (`get_formatted_uptime`, `get_battery_info`, `get_disks_info`).
  - Added compile-checked doc-tests for background/daemon guards (`set_low_priority`, `set_idle_priority`, `prevent_system_sleep`, `BackgroundPowerGuard`).

## [1.9.17] - 2026-06-07

### Added
- **First-class CLI Scaffold & Auto-routing**:
  - Enhanced `CliParser` with a `logo` builder (`CliParser::logo(...)`) to configure custom headers/banners on help screens.
  - Added `determine_scaffold_action(...)` returning a `ScaffoldAction` enum for testable argument routing.
  - Added process execution handlers `parse_env_args_or_exit(...)` and `parse_args_or_exit(...)` to automatically route common options (`--help` / `-h` / `help`), version flags (`--version` / `-v` / `version`), and system checks (`doctor`).
  - Documented `CliParser` with a rich `rustdoc` example.
- **Improved Test Suite**:
  - Added `test_determine_scaffold_action` verifying scaffold flag and command routing behavior without process exit.

## [1.9.16] - 2026-06-07

### Added
- **Fleshed out `lifecycle::background::daemon`**:
  - Implemented `set_low_priority` and `set_idle_priority` process-level and thread-level priority classes to cleanly yield CPU resources to user-facing applications.
  - Implemented `prevent_system_sleep` supporting Win32 `SetThreadExecutionState` power flags (`ES_SYSTEM_REQUIRED` | `ES_AWAYMODE_REQUIRED`) to keep the CPU awake during long-running background tasks while allowing the monitor to sleep.
  - Added an RAII-based `BackgroundPowerGuard` to acquire sleep prevention on construction and automatically release it when dropped.
  - Added full Unix FFI stubs for `setpriority` (niceness) controls to yield CPU priority on Linux.
- **Improved Test Suite**: Added `test_daemon_helpers` covering priority setting, sleep prevention flags, and RAII drop releases.

## [1.9.15] - 2026-06-07

### Added
- **CI Taxonomy Enforcement**: Added an integration test suite `taxonomy_compliance` that statically parses module references to enforce strict, unidirectional layering constraints (ensuring `core` is isolated, `platform` does not import `interface`/`lifecycle`/`role`, etc.).
- **Architectural Cleanup (Taxonomy Compliance)**:
  - Moved high contrast querying (`query_high_contrast`) from `lifecycle` to `platform::native::sys_info` (which holds all other display/theme properties) and re-exported it in `lifecycle::foreground::console` for backward compatibility.
  - Decoupled `core.rs` from platform-specific functions (`get_system_info`) by moving its implementation into the `platform` layer (`platform::native::sys_info`), converting `core` into a pure, dependency-free container layer.
  - Relocated `get_packages_breakdown` re-exports to prevent platform-layer coupling with high-level application package utilities.

### Fixed
- **Win32 Named Pipes Synchronization**: Added a Win32 `FlushFileBuffers` synchronization call in local IPC server responses to ensure the client has finished reading response data before the server disconnects the pipe. This resolves flaky `ERROR_PIPE_NOT_CONNECTED` (233) failures during parallel test execution.

## [1.9.14] - 2026-06-07

### Added
- **AccentTheme & Color Provider Helper**: Formalized `AccentColors` usage by adding dynamic system theme constructors (`AccentColors::query_system`, `AccentColors::calculate_from_accent`) and introducing the `AccentTheme` provider to query or construct standardized dark/light themes with zero boilerplate.
- **Improved Test Coverage**: Added tests verifying color calculations, fallback defaults, and system queries.

### Fixed
- **Resolved Heap Corruption**: Fixed a heap corruption bug (`STATUS_HEAP_CORRUPTION`) in Win32 `query_high_contrast` by removing an invalid `LocalFree` call on a system-allocated scheme string pointer.

## [1.9.13] - 2026-06-07

### Added
- **Expanded GUI Native Helpers**: Added cross-platform message boxes, error/warning dialogs, and Yes/No confirmation prompts in `interface::gui::native` utilizing native Win32 dialogs on Windows and interactive/stream fallbacks on other platforms.
- **Added Premium egui Widgets**: Created reusable glassmorphic layout wrappers and widgets (`AccentCard`, `AccentButton`, `AccentTabs`) in `interface::gui::helpers` matching the visual aesthetics of the local76 apps.
- **Added GUI Test coverage**: Introduced in-memory rendering verification tests for the new egui widgets to validate layout safety and style overrides.

## [1.9.12] - 2026-06-07

### Added
- **Strengthened Cross-Platform Stubs**: Fully implemented Web, Mobile, and Embedded stubs in `src/platform/` with specific mock responses for OS, display, power status, GPU, network adapters, and viewport size.
- **Refactored Fallback Dispatching**: Configured `sys_info.rs` and `monitors.rs` to dynamically route generic stubs to specific `web`, `mobile`, and `embedded` platform modules when compiling for those architectures/platforms.
- **Fleshed out `interface::cli`**: Implemented a lightweight, schema-based `CliParser` supporting subcommands, flags, options with/without values, and positional arguments. Added support for custom diagnostics check-ups via `run_doctor_with_custom` and upgraded `parse_cli_args` to match custom subcommands.
- **Fleshed out `interface::api` (Local IPC)**: Introduced cross-platform `IpcServer` and `IpcClient` abstractions utilizing native Win32 Named Pipes on Windows, Unix Domain Sockets on Linux/Unix, and simulated in-memory channels on other targets.
- **Added Focused/Active Support**: Added `is_console_focused()` to check if the console window currently has system focus, and added `active` / `focused` controls to `RgbController` and `ObstacleJumpGame` to pause work when inactive.
- **Expanded Test Suite**: Added unit tests for new stubs, CLI commands/options, Local IPC socket communication, `AccentTabs`, `AccentTextBox`, TUI effects, `get_all_monitors`, `RgbController`, and `formatting.rs` helpers. Increased total tests from 19 to 28.

## [1.9.10] - 2026-06-?? (everything first-class)

### Added / First-Class Improvements (s+)
- **AccentColors** (new first-class struct in `interface::tui::widgets`): Bundle `accent`, `dim`, `text` colors. All `Accent*` widgets now have `new_with_colors(...)` constructors for dramatically less boilerplate in consumers. Recommended way to theme entire panels consistently.
- `AccentGauge`, `AccentTabs`, `AccentList`, `AccentTextBox` now uniformly first-class with `focused: bool`:
  - Bright accent + bold + special indicators (bar, fill) **only** when focused.
  - Current/selected element remains visible (using text or dim colors) when the panel is inactive.
  - Matches the "active border + current selection visible" pattern used across r* apps.
- **Effects first-class**: `TuiEffect` trait and all major implementations (`MatrixRain`, `SimpleParticles`, `GravityParticles`, `RainEffect`, `FireEffect`) now have public `active: bool` (defaults true).
  - When `!active` (e.g. effect preview panel not focused): updates are no-ops, draws are minimal/cleared. Prevents wasted CPU and makes inactive previews look "paused".
  - All effects already fully respect dynamic `cols`/`rows` for resize (first-class for variable preview boxes).
  - Updated trait docs and effect structs with first-class usage notes.
- **Text helpers** (`interface::tui::text`): `wrap_text` + `align_line` + `TextAlignment` polished with better docs and examples. First-class for any custom TerminalCell / grid rendering.
- Comprehensive taxonomy + rustdoc improvements across **core**, **interface** (cli/gui/tui/api), **lifecycle** (foreground/background), **platform** (native + stubs), **role** (system/application). Every major module now has clear "first-class" guidance for reuse.
- `AccentColors::new` / `from_accent_dim_text` + `new_with_colors` helpers make the entire accent widget family trivial and consistent to use.
- Re-exports and lib.rs docs updated to surface the new first-class primitives.

### Changed
- All accent widgets and effects now encourage (and document) the focused/active pattern for tab/focus UIs without duplicating logic in every consumer.
- Widget constructors remain backward-compatible; the `_with_colors` variants are the new recommended first-class path.
- Effects draw/update now early-exit on `!active` for efficiency (while still allowing the caller to render a dimmed grid if desired).
- Module docs in `tui/mod.rs` and widget headers now explicitly call out "first-class for tab/focus-based UIs".

### Fixed / Polish
- Inconsistent color naming and focus handling across the old accent widgets eliminated.
- Effects no longer have hidden hardcoded sizes or always-on CPU usage when their container panel is inactive.
- Better discoverability of helpers (AccentColors, text utils, effects active flag).
- Continued cleanup of legacy patterns in favor of taxonomy-aligned, reusable code.

### Tests (first-class completeness with remaining credits)
- Added dedicated `#[cfg(test)]` module to `widgets.rs` covering:
  - `AccentColors` construction and `new_with_colors` paths.
  - Focused vs inactive construction for `AccentList`.
  - Basic render path execution for focused/inactive list states (validates first-class behavior).
  - `AccentGauge` with colors.
- Added tests in `effects.rs` for the new `active` flag:
  - Inactive effects skip updates (state preserved).
  - Default `active: true` on major effects.
- These directly exercise the "first class for tab/focus UIs" contract added in 1.9.9/1.9.10.

This release makes library's presentation and visual primitives truly first-class shared components. Consumers can now build sophisticated focus-aware TUIs (lists, tabs, gauges, live effects previews) with minimal code and guaranteed visual/behavioral consistency.

## [1.9.9] - 2026-06-?? (first-class TUI widgets)

### Added
- `focused: bool` parameter to `AccentList`, `AccentTabs`, and `AccentGauge` (and existing on `AccentTextBox`).
- When `focused=true` the widget uses full accent color + bold for the active/selected element (and underline bar for tabs, fill for gauge).
- When `focused=false` (inactive panel) the "current" selection remains visible using dim or main text colors (no bright accent, no bar) so consumers can keep a consistent "current item" indication while the whole panel de-emphasizes together with `border_active` borders.
- This makes the entire accent widget family first-class for tab/focus-based UIs (e.g. cycling between Diagnostics/Effects panels, lists that must stay in sync with details views, etc.).
- Updated module-level docs in `interface::tui` to highlight the first-class focused widgets pattern.

### Changed
- `AccentList` selected row now uses configurable `active_text_color` (instead of always forcing bold accent on item when focused) for better flexibility.
- Gauge fill and text overlay now respect `focused` (falls back to dim when inactive).
- Tabs only draw the accent underline bar when both selected *and* focused.
- All widgets remain zero-allocation where possible and render into the supplied `Rect` (typical usage: draw your `Block` with conditional `border_active`, then render the accent widget into `block.inner(area)`).

### Fixed
- Previous manual prefix/indent/color duplication between lists (e.g. "  " vs "▶ ") is now centralized and guaranteed identical when using the widgets.

## [1.9.6] - 2026-06-07

### Added
- Full taxonomy-aligned module structure: core, interface (cli, tui, gui, api), lifecycle (foreground, background), platform (native), role (system, application).
- Comprehensive CLI helpers in `interface::cli` (arg parsing, help/version, doctor/diagnostics patterns generalized from template, helm, trance).
- Expanded TUI effects: generalized cosmos (gravity/particles), storm, flame, bursts, beams, chaos from trance-scenes into `interface::tui::effects`.
- GUI Native support stubs in `interface::gui::native` (basic cross-platform dialog/message helpers).
- Expanded Headless/API in `interface::api` with concrete examples (IPC traits, service API facades, serialization helpers for DashboardInfo).
- More lifecycle background: power management, thread priority helpers for daemons.
- Platform expansions: web (wasm stubs), mobile (stubs), embedded (stubs); more complete cross-platform fallbacks in sys_info and monitors.
- Full cross-platform `get_system_info` with static/dynamic logic from trance-core (Windows registry, Linux /proc parsing).
- Fleshed out `role::system` with low-level infrastructure (power, event log facades, etc.).
- Additional formatting/info helpers from helm (get_battery_info, get_memory_info, get_disks_info, get_formatted_uptime, get_host_info, get_cpu_info, get_gpu_names, detect_shell_and_terminal) in `role::application`.
- rpack enhancements: more general packaging abstractions.
- Extensive documentation: classification comments in all modules, rustdoc examples, updated ARCHITECTURE.md with full audit and gaps.
- Comprehensive tests for new modules (core, effects, packages, monitors, cli, api, etc.).
- CHANGELOG.md and updated version to 1.9.6 reflecting major reorganization and feature completeness.

### Changed
- Reorganized source to strictly follow the 4-layer taxonomy to prevent coupling.
- Updated features in Cargo.toml with taxonomy-aligned ones (interface-tui, etc.) while maintaining legacy compat.
- Enhanced core to be truly neutral single source of truth.
- Improved cross-platform support across the board.
- Backward compat maintained via re-exports and legacy win32 shim.

### Fixed
- Various duplications and platform bleed from consumer projects now centralized.
- Minor compile warnings and unused imports cleaned during audit.

## [1.9.5] - Previous (reorg baseline)

Major reorganization into taxonomy modules, ports from other projects (effects, SystemInfo, packages, monitors, console helpers, etc.), feature alignment, initial docs.

See git history for details.