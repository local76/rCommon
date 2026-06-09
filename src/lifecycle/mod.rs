//! Execution State (Lifecycle)
//!
//! **Taxonomy Classification**: Execution State (Lifecycle).
//!
//! How the operating system manages the application's runtime.
//!
//! Categories:
//! - Foreground Applications (require active user attention and focus) - see foreground/
//! - Background Processes (run silently, often at startup, no interface needed) - see background/
//!
//! For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/ARCHITECTURE.md).
//! Cross-platform with native features and platform-specific stubs.

pub mod foreground;
pub mod background;