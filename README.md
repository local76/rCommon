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
You can run any screensaver or effect polymorphically using the `TuiEffect` trait:

```rust
use rcommon::interface::tui::effects::{MatrixRain, TuiEffect};
use rcommon::core::TerminalCell;

let mut effect = MatrixRain::new(80, 25, 0.1);
let mut grid = vec![TerminalCell::default(); 80 * 25];

// Game/render loop:
effect.update(0.016, 80, 25);
effect.draw(&mut grid, 80, 25);
```

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

---

## 📄 License

This project is licensed under the MIT License. See [LICENSE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/rCommon/LICENSE.md) for details.
