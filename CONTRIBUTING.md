# Contributing to library

Thank you for your interest in contributing to `library`! This document outlines the guidelines and best practices for developing, testing, and contributing to the shared foundation of the `local76` ecosystem.

---

## 🏛️ Architecture & Taxonomy

All code added to `library` must conform to the 4-layer taxonomy defined in [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/ARCHITECTURE.md). This classification prevents accidental coupling between distinct concerns.

### The 4 Layers

1. **Interface (Presentation Layer)**:
   - Location: `src/interface/`
   - Purpose: Visual/non-visual communication (CLI, TUI, GUI, Headless API).
   - Rules: Keep presentation logic separate from system operations.

2. **Execution State (Lifecycle)**:
   - Location: `src/lifecycle/`
   - Purpose: OS-level execution status (Foreground console management, silent background services/daemons).
   - Rules: Avoid adding specific application logic here; focus on execution boundaries.

3. **Platform & Architecture (Deployment)**:
   - Location: `src/platform/`
   - Purpose: OS-specific integrations (Windows Registry, DRM display connector queries, etc.).
   - Rules: Restrict host-specific APIs to `platform/native/{windows, linux}.rs` and expose them through unified cross-platform interfaces in `platform/native/sys_info.rs`.

4. **System Role (Purpose)**:
   - Location: `src/role/`
   - Purpose: Purpose-driven operations (low-level system infrastructure in `role/system`, user-oriented application logic in `role/application`).

*Note: Truly neutral primitives (data shapes, deterministic RNGs, etc.) that have zero heavy platform or presentation assumptions may be placed in [src/core.rs](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/src/core.rs).*

---

## 📦 Feature Gates & Binary Size

To prevent bloat in small CLI daemons, all new functionality must be gated behind appropriate Cargo features.

- **Prefer taxonomy-aligned features** (e.g. `interface-tui`, `platform-native`, `role-application`) for consumer-facing imports.
- Place all raw library dependencies (like `ratatui`, `eframe`, `sysinfo`) under optional dependencies and gate them using granular feature flags in `Cargo.toml`.

---

## 🛠️ Cross-Platform Standards

`library` supports both **Windows** and **Linux** targets.
- Ensure that any code utilizing raw system APIs (such as Win32 or Linux `/sys`/`/proc`) has appropriate `#[cfg(target_os = "...")]` guard rails.
- Provide clean cross-platform fallbacks or stubs for other operating systems to prevent compilation failures.

---

## 🧪 Testing Guidelines

Every new module or helper must include unit tests.

### Running Tests
Run the entire test suite across all features and targets before submitting a pull request:
```bash
cargo test --all-features
```

Verify that compilation passes under all workspace configurations:
```bash
cargo check --all-targets --all-features
```

---

## 📥 Pull Request Process

1. **Fork & Branch**: Create a feature branch from `main`.
2. **Implement**: Write code conforming to the taxonomy, add docstrings, and implement tests.
3. **Audit**: Run cargo checks and tests.
4. **Document**: Update the [CHANGELOG.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/CHANGELOG.md) under the `[Unreleased]` section if making any user-facing API changes.
5. **Submit**: Create a PR targeting `main` at `https://github.com/local76/library/pulls`.
