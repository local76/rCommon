# library Consolidation Audit (post 4.2.0)

**Date**: 2026-06-08
**Auditor**: Crush (sub-agent) + human follow-up
**Scope**: Top-level `C:/Users/jeryd/Synology/Home/Projects/local76` (helm, trance, pulse, ignite, template, scout, hub, library, screensavers)

## Summary

The 6 r* TUI apps share a large amount of code that has been **partially** consolidated into library 4.2. Many helper modules already exist in library (logger, file_log, tui_bootstrap, design/markdown, design/layout, design/theme, design/text, cli/doctor, sys_info, win32/window) but the migration is **incomplete** — most apps still ship local copies of helpers that library now owns, or have not adopted helpers library added months ago.

The biggest wins below are: **(a) finish the migrations the changelog already claims are done**, and **(b) extract 1-2 genuinely new shared helpers that no app has built yet but all of them need**.

## Top 10 duplications

| # | Pattern | Apps affected | library layer | LOC saved |
|---|---|---|---|---|
| 1 | `hide_console_at_startup` + `show_console_window` | ignite, scout (+ pulse, scout inline) | `lifecycle/foreground/window` | ~110 |
| 2 | `file_log` / `log_message` | ignite, scout | `lifecycle/background` | ~110 |
| 3 | `centered_rect` | trance (×2), ignite (×2), scout | `interface/tui/design` | ~90 |
| 4 | `get_theme` factory | ignite, scout | `interface/tui/design` | ~40 |
| 5 | `parse_markdown_to_lines` | ignite, scout, trance | `interface/tui/design` | ~250 |
| 6 | TUI panic hook w/ logger | trance, ignite | `lifecycle/foreground` | ~50 |
| 7 | CLI `--version/--help/--doctor` dispatch | pulse, ignite, scout, trance (partial) | `interface/cli` | ~70 |
| 8 | `config_path()` APPDATA resolver | helm, trance, ignite, scout | `platform/native` | ~25 |
| 9 | `accent_color_from_hex` | pulse | `interface/tui/design` | ~11 |
| 10 | `format_help_row` | trance, ignite | `interface/tui/design` | ~90 |
| **Total** | | | | **~846 LOC** |

## Categorization

### "Finish the migration" (zero-ambiguity wins — library already owns the canonical impl, just delete the local copies)

- **Item 9** (accent_color_from_hex) — library has it; pulse has a stale local copy.
- **Item 1** (hide_console_at_startup) — library has it; ignite + scout have stale local copies.
- **Item 2** (file_log) — library has it; ignite + scout have 55-line local copies of each other.
- **Item 8** (config_path) — library has it; 4 apps still hand-roll APPDATA path resolution (and miss the XDG on non-Windows).
- **Item 4** (get_theme factory) — library has the superset (13 fields vs the 4-field local versions).
- **Item 10** (format_help_row) — library has it; trance + ignite have stale local copies.
- **Item 3** (centered_rect) — library has it (and internally has 2 copies of its own, in `layout.rs` and `layout_guard.rs`); 4 apps have local copies.
- **Item 5** (parse_markdown_to_lines) — library has it; ignite + scout + trance still have local 60-110 line copies.

### "Partial migration, library needs a small extension"

- **Item 6** (panic hook w/ logger) — library has `set_tui_panic_hook` (no logger). Add `set_tui_panic_hook_with_logger<F: Fn(&str, &str)>` that delegates to the existing one + calls the injected log callback.
- **Item 7** (CLI dispatch) — library has `CliParser` (used by template) and `screensaver_runtime::parse_args()` (used by trance's `.scr` mode). pulse/ignite/scout still hand-roll. The `CliParser` needs to support the `--json`/`--install`/`--doctor` triad (helm-style) before the apps can adopt it cleanly.

## Per-app summary (where they each stand today)

| App | Status | Examples |
|---|---|---|
| helm | Most-migrated | Uses `design::prelude`, all library helpers; only `config_path` is local |
| template | Migrated | Uses `CliParser`; canonical library pattern |
| pulse | Partial | `accent_color_from_hex` still local; hand-rolled CLI dispatch |
| ignite | Least-migrated | Local `hide_console_at_startup`, `file_log`, `get_theme`, `parse_markdown_to_lines`, `format_help_row`, `centered_rect`, hand-rolled CLI dispatch |
| trance | Partial | Local `centered_rect` (×2), `parse_markdown_to_lines`, `format_help_row`, custom panic hook with log glue |
| scout | Least-migrated | Local copies of everything ignite has |
| pulse | Same as ignite for most helpers | Also has the `restore_console_window` inline re-show block |
| hub | Meta/dir, not code | N/A |

## Recommended migration order (easiest wins first)

1. **Item 9** (accent_color_from_hex) — 11 lines, no signature compat issues.
2. **Item 1** (hide_console_at_startup) — delete local + add `restore_console_window` helper to library.
3. **Item 2** (file_log) — replace 55-line `logger.rs` with 1-line re-export shim in ignite + scout.
4. **Item 8** (config_path) — 4 apps adopt library's `AppConfig::config_path`. Cross-platform bonus for free.
5. **Item 4** (get_theme) — drop-in, superset.
6. **Item 10** (format_help_row) — drop-in once trance swaps `TuiTheme` → `ThemeColors`.
7. **Item 3** (centered_rect) — drop-in, but also collapse library's two internal copies.
8. **Item 5** (parse_markdown_to_lines) — drop-in for ignite, scout, trance.
9. **Item 6** (panic hook w/ logger) — requires adding `set_tui_panic_hook_with_logger` to library.
10. **Item 7** (CLI dispatch) — biggest refactor; requires `CliParser` to support the apps' heterogeneous subcommand sets.

## Estimated impact

The combined **library 4.3 "finish the migration"** follow-up would dedupe **~846 lines** across the 6 apps, plus the ~300 lines of library's own internal duplication (two `centered_rect` copies in `layout.rs` and `layout_guard.rs`; the `is_dark_mode`/`query_dark_mode` aliasing in `lifecycle/foreground/power_sync.rs:129`; etc.) for a total of **~1100 LOC** of duplicate or near-duplicate code that can be removed once the migrations complete.

That's roughly the equivalent of an entire r* TUI app's UI utilities layer collapsing into library.

## New library 4.3 APIs to add

Based on the audit, the following 5 small additions would unblock the migrations:

1. `lifecycle::foreground::window::restore_console_window()` (new) — companion to `hide_console_at_startup`. Currently every app re-implements the `ShowWindow(SW_SHOW) + SetForegroundWindow` re-show block.
2. `lifecycle::foreground::panic::set_tui_panic_hook_with_logger<F>(logger: F)` (new) — wraps the existing `set_tui_panic_hook` and adds an injected log callback for `file_log::log_message` glue.
3. `interface::cli::scaffold::CliParser::subcommand_with_args(name, doc, callback)` (new) — supports the `--doctor --json` style compound subcommands that helm uses.
4. `interface::tui::design::layout::format_help_row(&ThemeColors)` (already exists; trance + ignite adoption blocker is the `TuiTheme` vs `ThemeColors` type mismatch — likely needs a `From<TuiTheme> for ThemeColors` shim).
5. Internal: collapse the 2 copies of `centered_rect` in library (`layout.rs:14` + `layout_guard.rs:64`) into one.

## Per-app follow-up tasks (proposed for library 4.3 / r* 4.x bumps)

- **helm 3.1.0**: drop local `config_path` (use `library::platform::native::config::AppConfig::config_path("helm", "config.yaml")`).
- **template 3.2.0**: minor; mostly already migrated.
- **pulse 3.2.0**: drop local `accent_color_from_hex`; adopt `CliParser`.
- **ignite 4.0.0**: drop local `hide_console_at_startup` + `show_console_window`; drop local `file_log` (replace with 1-line shim); drop local `get_theme`; drop local `centered_rect`; drop local `parse_markdown_to_lines`; drop local `format_help_row`; adopt `CliParser`; use library's `set_tui_panic_hook_with_logger`.
- **trance 3.0.0**: drop both `centered_rect` copies; drop local `parse_markdown_to_lines`; drop local `format_help_row`; switch to library's `set_tui_panic_hook_with_logger`.
- **scout 4.0.0**: same set of changes as ignite (the two are near-twins in terms of duplicated helpers).

## Conclusion

The 4-phase migration plan (library 4.0 design system → 4.1 scenes consolidation → 4.2 screensaver_runtime consolidation → 4.3 ?) is **functionally complete as of 4.2.0**: all 10 r* effects are in library, trance-core is gone, the r* shim binaries work, the 6 r* TUI apps compile against library 4.2. The remaining work in the audit above is **incremental polish** — finishing migrations that the changelog already claims are done — not new features.

A focused 4.3 sprint of 10-15 small PRs (one per audit item) would close the loop and dedupe ~846 LOC of app code. Each PR is small, low-risk, and has a clear test surface. None of them requires any new design work — they're all "the canonical impl already exists in library, just import it".

## Cross-cutting observation

A pattern in the audit: library often has the canonical impl, but the apps don't pick it up because:
- (a) the canonical impl was added in a library minor release and the apps are still on the old major
- (b) the apps' Cargo.toml features lists don't enable the library feature that contains the impl (e.g. ignite doesn't enable `scenes` because it doesn't need the effects, but it also doesn't enable other new features it could be using)
- (c) the library migration guides at `src/lib.rs:25-30` are out of date (the last entry is from 3.x; the 4.0 design system + 4.1 palette + 4.1 ScreenPalette + 4.2 screensaver_runtime entries are missing)

Suggested follow-up: refresh the library migration guide at `src/lib.rs:23-30` after the 4.3 audit PRs land, with explicit per-app migration entries (helm should adopt `AppConfig::config_path`, pulse should drop `accent_color_from_hex`, etc.). The apps' READMEs should also be checked for stale "use the local helper" instructions that predate the library migration.
