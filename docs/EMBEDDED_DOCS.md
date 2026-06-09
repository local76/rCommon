# Embedding markdown docs in your TUI app (F1–F7 in-TUI help)

`library::embedded_docs!` is a compile-time macro that bakes a set of markdown files (your README, LICENSE, CONTRIBUTING, etc.) directly into your binary, so your TUI can show the help text without reading the filesystem at runtime (which would break in a single-file `.scr` Windows screensaver install).

## The macro

`library::interface::tui::design::markdown::embedded_docs!(folder, [file1, file2, ...])` — declared as `#[macro_export]`, available from any consumer as `library::embedded_docs!(...)`.

Internally it is a thin wrapper over `include_str!` that returns a `HashMap<&'static str, &'static str>` mapping file names to contents. The compiler reads the files at build time; nothing reads the filesystem at runtime.

## Canonical example (from `helm`, the reference implementation)

```rust
use std::collections::HashMap;
use std::sync::LazyLock;

/// Standardized embedded documents map.
pub static EMBEDDED_DOCS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        library::embedded_docs!("..", [
            "README.md",
            "SUPPORT.md",
            "LICENSE.md",
            "COPYRIGHT.md",
            "PRIVACY.md",
            "SECURITY.md",
            "CONTRIBUTING.md",
        ])
    });
```

(`".."` is relative to the manifest dir of the consuming crate; e.g. for `helm` at `local76/helm/crates/helm/`, `".."` would be `local76/helm/crates/`. Adjust to your layout.)

## Wiring it into a TUI panel

The standard pattern in `library`'s TUI apps is to bind F1–F7 to the embedded docs and render the selected doc with `library::interface::tui::design::markdown::MarkdownViewer`:

```rust
use crossterm::event::{KeyCode, KeyEvent};
use library::interface::tui::design::markdown::MarkdownViewer;

fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::F(1) => app.active_doc = Some("README.md"),
        KeyCode::F(2) => app.active_doc = Some("SUPPORT.md"),
        KeyCode::F(3) => app.active_doc = Some("LICENSE.md"),
        // ... F4..F7 for COPYRIGHT/PRIVACY/SECURITY/CONTRIBUTING
        KeyCode::Esc => app.active_doc = None,
        _ => {}
    }
    if let Some(name) = app.active_doc {
        if let Some(text) = EMBEDDED_DOCS.get(name) {
            app.viewer = MarkdownViewer::new(text);
        }
    }
}
```

(See `library/interface/tui/design/markdown_viewer.rs` for the viewer API. The 5 TUI apps each wire a subset of F1–F7 to their embedded docs.)

## Notes

- The macro uses `include_str!` under the hood, so the path is resolved at compile time. A typo in the file name = a compile error, not a runtime fallback. That's intentional: you want to catch a missing file at `cargo build` time, not at first launch.
- The macro is intentionally simple — no glob, no directory walk. Explicit file lists are preferred so the compiler knows exactly what's in the binary.
- The macro is in `library::interface::tui::design::markdown` (not in the deprecated `interface::tui` paths). Use the canonical path.

## Why this exists

Prior to `library` 4.0, every TUI app in the suite had its own F1–F7 help panel implementation, with different in-memory string tables and different render paths. The 4.0 design-system consolidation moved all of them to a single `library::interface::tui::design` façade + this `embedded_docs!` macro + a single `MarkdownViewer` renderer. The 4.0 → 4.1 → 4.2 evolution has been about extending this pattern (e.g. the scene shim binaries in `screensavers` do not use it because they are too small to need in-TUI help).

## See also

- `library::interface::tui::design::markdown::MarkdownViewer` (the renderer that consumes the `&'static str` content)
- `library::interface::tui::design::markdown::parse_markdown_to_lines` (lower-level; used internally by `MarkdownViewer`)
- Each app's `event_handler.rs` for the F1–F7 binding pattern
