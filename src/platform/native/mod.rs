//! Native platform implementations.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment - Native).
//!
//! Part of Platform & Architecture (Deployment).
//! OS-specific code for Windows and Linux native targets.

// Native platform code moved here.

pub mod sys_info;
pub mod reg;
pub mod monitors;  // Monitor enumeration from helm/pulse patterns.
pub mod config;

#[cfg(feature = "gpu")]
pub mod gpu;

// Note: The sys_info/ subdirectory (with linux.rs/windows.rs) provides the
// platform-specific implementations for the sys_info module.
