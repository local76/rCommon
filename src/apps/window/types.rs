//! Common type definitions for window and console layouts.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct MONITORINFO {
    pub cbSize: u32,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: u32,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct COORD {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct SMALL_RECT {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct CONSOLE_SELECTION_INFO {
    pub dwFlags: u32,
    pub dwSelectionAnchor: COORD,
    pub srSelection: SMALL_RECT,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}
