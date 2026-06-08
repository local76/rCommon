# Changelog

All notable changes to rCommon will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.0.16] - 2026-06-08

### Added
- **Theme-Aware Selection Foreground**: Added `selection_fg` to `ThemeColors` to support theme-customized selection highlight colors in TUI interfaces.

## [3.0.15] - 2026-06-08

### Fixed
- **Clippy `too_many_arguments` on `draw_title_banner`**: Annotated the title banner widget with `#[allow(clippy::too_many_arguments)]` plus a justification comment, since the function intentionally composes the full title strip (title, version, user, host, OS) in a single render pass.
- **Clippy `type_complexity` in `sys_info` caches**: Introduced `CachedAccent`, `CachedBool`, `CachedString`, `CachedTheme`, and `CachedPower` type aliases in `platform::native::sys_info` to clarify the tuple-of-`(Instant, T)` cache entries used by the TTL-bounded query helpers.
- **Clippy `type_complexity` in `logo` block cache**: Introduced a `LogoCacheEntry` type alias in `interface::tui::effects::logo` to document the `(text, sub_text, rendered_lines)` tuple cached by `render_logo_block`.
- **Clippy `module_inception` for `interface::gui::gui`**: Renamed the inner `gui` submodule to `egui_helpers` (file `gui.rs` → `egui_helpers.rs`) so the parent `interface::gui` module no longer contains a child module of the same name. Top-level `rcommon::gui` re-export still resolves to the same helpers via the renamed module.

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
- **Title Banner Widget**: Created `rcommon::interface::tui::widgets::title_banner` containing `draw_title_banner` for rendering standard TUI app title strips.
- **Effect Preview Widget**: Created `rcommon::interface::tui::widgets::effect_preview` containing `draw_effect_preview` for displaying screensaver grids.

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
- **Legacy Win32 Shim & Feature**: Deleted the deprecated, flat `rcommon::win32` compatibility module from `src/lib.rs` and its associated `win32` meta-feature from `Cargo.toml`. This is a breaking change completing the transition to the new 4-layer taxonomy modules.

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
  - Refactored `SingleInstanceGuard::try_new` in `lifecycle::foreground::guard` to return `crate::error::Result<Self>` wrapping errors in the custom `RcommonError::Guard` variant instead of returning raw `String` errors.
  - Refactored all local IPC APIs and wrappers (`IpcServiceHost`, `IpcServiceClient`, `IpcServer`, `IpcClient`) under `interface::api` to return `crate::error::Result` utilizing `RcommonError::Ipc` instead of returning standard `std::io::Error`.
  - Refactored the internal OpenRGB protocol device parser `parse_device_payload` under `role::application::rgb::protocol` to return `crate::error::Result` utilizing `RcommonError::Rgb` instead of returning static `&'static str` errors.
  - Introduced `RcommonError::is_ipc_termination` to easily inspect and identify transient connection terminations or client disconnect signals cleanly across platforms.

## [1.9.21] - 2026-06-08

### Added
- **Reusable Headless & IPC Services**:
  - Implemented structured request/response message formats (`IpcRequest`, `IpcResponse`) supporting direct serialization/deserialization.
  - Added a reusable background thread event loop handler (`IpcServiceHost`) and client client wrapper (`IpcServiceClient`) to avoid duplicate logic across background daemons (e.g. `rStartup`, `rIdle`, `rTemplate`).
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

This release makes rCommon's presentation and visual primitives truly first-class shared components. Consumers can now build sophisticated focus-aware TUIs (lists, tabs, gauges, live effects previews) with minimal code and guaranteed visual/behavioral consistency.

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
- Comprehensive CLI helpers in `interface::cli` (arg parsing, help/version, doctor/diagnostics patterns generalized from rTemplate, rFetch, rIdle).
- Expanded TUI effects: generalized rLife (gravity/particles), rPour, rFire, rFireworks, rBeams, rUnstable from rIdle-scenes into `interface::tui::effects`.
- GUI Native support stubs in `interface::gui::native` (basic cross-platform dialog/message helpers).
- Expanded Headless/API in `interface::api` with concrete examples (IPC traits, service API facades, serialization helpers for DashboardInfo).
- More lifecycle background: power management, thread priority helpers for daemons.
- Platform expansions: web (wasm stubs), mobile (stubs), embedded (stubs); more complete cross-platform fallbacks in sys_info and monitors.
- Full cross-platform `get_system_info` with static/dynamic logic from ridle-core (Windows registry, Linux /proc parsing).
- Fleshed out `role::system` with low-level infrastructure (power, event log facades, etc.).
- Additional formatting/info helpers from rFetch (get_battery_info, get_memory_info, get_disks_info, get_formatted_uptime, get_host_info, get_cpu_info, get_gpu_names, detect_shell_and_terminal) in `role::application`.
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