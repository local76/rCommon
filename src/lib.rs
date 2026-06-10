// =====================================================
// library - Shared utility library for the local76 apps ecosystem
// Reorganized into a simplified flat folder tree:
// 1. core/ (neutral foundation)
// 2. ui/ (widgets & design)
// 3. toolkit/ (platform & deployment)
// 4. apps/ (controllers & lifecycle)
// =====================================================

// Simplified modules
pub mod core;
pub mod ui;
pub mod toolkit;
pub mod apps;

#[cfg(feature = "screensaver-runtime")]
pub mod screensaver_runner;

// Re-export error and main traits
pub mod error {
    pub use crate::core::error::*;
}
pub use error::{LibraryError, Result as LibraryResult};

#[cfg(feature = "effects")]
pub use ui::screensaver_renderer::{Screensaver, ScreensaverRenderer};

/// Extension trait to expose background daemon services over IPC.
#[cfg(feature = "interface-api")]
pub trait DaemonIpcExt {
    fn run_ipc_server<F>(&self, handler: F) -> Result<(), crate::core::error::LibraryError>
    where
        F: Fn(toolkit::ipc::IpcRequest) -> toolkit::ipc::IpcResponse;
}

#[cfg(all(feature = "interface-api", feature = "service"))]
impl DaemonIpcExt for apps::daemon::DaemonService {
    fn run_ipc_server<F>(&self, handler: F) -> Result<(), crate::core::error::LibraryError>
    where
        F: Fn(toolkit::ipc::IpcRequest) -> toolkit::ipc::IpcResponse
    {
        let host = toolkit::ipc::IpcServiceHost::new(self.name())?;
        host.run(handler);
        Ok(())
    }
}
