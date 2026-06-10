//! Backward compatibility shim for lifecycle.
//! Re-exports from the new app and backend modules.

pub mod foreground {
    #[cfg(feature = "window")]
    pub use crate::apps::console;
    #[cfg(feature = "window")]
    pub use crate::apps::guard;
    #[cfg(feature = "sys-info")]
    pub use crate::apps::identity;
    #[cfg(feature = "lifecycle-foreground")]
    pub use crate::apps::panic;
    #[cfg(feature = "lifecycle-foreground")]
    pub use crate::apps::panic::set_tui_panic_hook;
    #[cfg(feature = "lifecycle-foreground")]
    pub use crate::apps::power_sync;
    #[cfg(feature = "widgets")]
    pub use crate::apps::tui_bootstrap;
    #[cfg(feature = "window")]
    pub use crate::apps::window;

    #[cfg(feature = "window")]
    pub use crate::apps::window::WindowDrag;
}

pub mod background {
    #[cfg(feature = "service")]
    pub use crate::apps::daemon;
    #[cfg(feature = "service")]
    pub use crate::apps::service;
    #[cfg(feature = "service")]
    pub use crate::apps::worker;
    #[cfg(feature = "event-log")]
    pub use crate::apps::event_log;
    #[cfg(feature = "lifecycle-background")]
    pub use crate::apps::file_log;
    #[cfg(feature = "notification")]
    pub use crate::apps::notification;
    #[cfg(feature = "clipboard")]
    pub use crate::toolkit::clipboard;
}

#[cfg(feature = "lifecycle-foreground")]
pub use foreground::panic::set_tui_panic_hook;
