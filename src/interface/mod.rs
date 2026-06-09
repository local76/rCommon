//! Interface (Presentation Layer)
//!
//! **Taxonomy Classification**: Interface (Presentation Layer).
//!
//! How the software communicates visually (or non-visually) with the user or other software.
//!
//! This module groups everything under category 1 of the library taxonomy:
//! - CLI (Command Line Interface)
//! - TUI (Text User Interface)
//! - GUI: Native/OS (Standard WIMP interfaces)
//! - GUI: Custom/Game Engine (Continuous-loop canvas environments)
//! - Headless / API (no UI, communicates with other software)
//!
//! ### Key Distinctions
//!
//! - **CLI**: Line-oriented stdio, sequential output, often stateless per invocation.
//! - **TUI**: Grid-based, in-place updates, stateful widgets/navigation, keyboard-driven.
//! - **GUI**: Pixel canvas, continuous 2D space, rich mouse input, higher visual fidelity.

#[cfg(feature = "interface-api")]
pub mod api;   // Headless / API (the piece added for the taxonomy)
pub mod tui;   // TUI widgets and effects (dir with mod.rs)
pub mod gui;   // GUI (native and custom/game engine)
pub mod cli;   // CLI (Command Line Interface)

// For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/ARCHITECTURE.md).
// Cross-platform with native features and platform-specific stubs.