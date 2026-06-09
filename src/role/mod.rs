//! System Role (Purpose)
//!
//! **Taxonomy Classification**: System Role (Purpose).
//!
//! The software's ultimate objective within the computing environment.
//!
//! Categories:
//! - System Software (infrastructure: manages hardware, provides platform — drivers, OS services, etc.) - see system/
//! - Application Software (task-oriented for end users or higher-level tools) - see application/
//!
//! For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/library/ARCHITECTURE.md).
//! Cross-platform with native features and platform-specific stubs.

// System Software components are low-level platform and lifecycle abstractions 
// implemented directly in platform/ (e.g., native::reg, native::sys_info) and 
// lifecycle/ (e.g., background::service). A separate re-export module under role/ 
// is redundant.


pub mod application;