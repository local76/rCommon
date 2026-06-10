//! Backend operations and platform-specific FFI utilities.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment).

#[cfg(feature = "reg")]
pub mod registry;

pub mod monitors;

#[cfg(feature = "gpu")]
pub mod gpu;
#[cfg(feature = "gpu")]
pub mod wgpu_renderer;

pub mod ebpf;

pub mod config;

#[cfg(feature = "clipboard")]
pub mod clipboard;

#[cfg(feature = "interface-api")]
pub mod ipc;
#[cfg(feature = "interface-api")]
pub mod ipc_messages;

#[cfg(feature = "sys-info")]
pub mod packages;



pub mod platform;
pub mod platform_embedded;
pub mod platform_mobile;
pub mod platform_web;

#[cfg(feature = "sys-info")]
pub mod sys_info;

