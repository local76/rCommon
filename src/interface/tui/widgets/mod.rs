//! First-class custom TUI widgets (gauges, lists, textboxes, tabs).
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).
//!
//! # Focus & Active States
//! - **Focused**: Controls visual emphasis. Focused widgets render with active accent styling (active colors, bold, bright text). Unfocused widgets render with muted, dimmed borders and indicators to preserve visual hierarchy in tab/focus layouts.

pub mod colors;
pub mod gauge;
pub mod list;
pub mod scrollbar;
pub mod tabs;
pub mod textbox;
pub mod toast;

pub use colors::{AccentColors, AccentTheme};
pub use gauge::AccentGauge;
pub use list::AccentList;
pub use scrollbar::AccentScrollbar;
pub use tabs::AccentTabs;
pub use textbox::{AccentTextBox, TextBox};
pub use toast::{ToastBox, ToastKind};
