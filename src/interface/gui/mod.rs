//! GUI (Graphical User Interface) components.
//!
//! **Taxonomy Classification**: Interface (GUI).
//!
//! Part of the Interface (Presentation Layer).
//! Includes both Native/OS GUIs and Custom/Game Engine continuous canvas UIs.

#[cfg(feature = "gui")]
pub mod egui_helpers;  // eframe/egui widget helpers (AccentCard, AccentButton, AccentTabs, get_default_options, apply_glassmorphism_style)
pub mod native;  // Native/OS GUI stubs (message boxes, etc.)