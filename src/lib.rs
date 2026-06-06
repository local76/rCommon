#[cfg(target_os = "windows")]
pub mod win32;

#[cfg(not(target_os = "windows"))]
#[path = "win32_stub.rs"]
pub mod win32;

#[cfg(target_os = "windows")]
pub mod reg;

#[cfg(not(target_os = "windows"))]
#[path = "reg_stub.rs"]
pub mod reg;

pub mod widgets;
