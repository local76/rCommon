//! Backward compatibility shim for screensaver_runtime.
//! Re-exports from the new screensavers::runtime module.

#[cfg(feature = "screensaver-runtime")]
pub use crate::screensavers::runtime::*;
