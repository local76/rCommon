//! Window and terminal management for foreground applications.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

pub mod types;
pub mod drag_to_move;

#[cfg(target_os = "windows")]
mod window_win;
#[cfg(target_os = "windows")]
pub use window_win::*;

#[cfg(target_os = "linux")]
mod window_linux;
#[cfg(target_os = "linux")]
pub use window_linux::*;

#[cfg(target_os = "macos")]
mod window_macos;
#[cfg(target_os = "macos")]
pub use window_macos::*;

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
mod window_fallback;
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub use window_fallback::*;

pub use types::{
    RECT, MONITORINFO, COORD, SMALL_RECT, CONSOLE_SELECTION_INFO, POINT,
};

// Re-export SingleInstanceGuard from guard module to preserve API compatibility
pub use crate::apps::guard::SingleInstanceGuard;
pub use drag_to_move::WindowDrag;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_types_impls() {
        let rect = RECT::default();
        assert_eq!(rect.left, 0);
        assert_eq!(rect.right, 0);

        let pt1 = POINT { x: 10, y: 20 };
        let pt2 = pt1;
        assert_eq!(pt1, pt2);

        let coord = COORD::default();
        assert_eq!(coord.x, 0);

        let small_rect = SMALL_RECT::default();
        assert_eq!(small_rect.left, 0);

        let sel_info = CONSOLE_SELECTION_INFO::default();
        assert_eq!(sel_info.dwFlags, 0);

        let monitor_info = MONITORINFO::default();
        assert_eq!(monitor_info.cbSize, 0);
    }
}

