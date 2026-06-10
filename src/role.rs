//! Backward compatibility shim for role.
//! Re-exports from the new backend, app, and scenes modules.

pub mod application {
    #[cfg(feature = "sys-info")]
    pub use crate::toolkit::packages;

    #[cfg(feature = "role-application")]
    pub use crate::apps::game;
    pub use crate::core::screen_palette as palette;
    pub use crate::core::formatting;
    

}
