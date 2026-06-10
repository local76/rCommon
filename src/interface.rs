//! Backward compatibility shim for interface.
//! Re-exports from the new ui and app modules.

#[cfg(feature = "widgets")]
pub mod app {
    #[cfg(feature = "effects")]
    pub use crate::ui::effects;
    pub use crate::ui::screensaver_renderer as screensaver;
    pub use crate::ui::status_bar::StatusBar;
    pub use crate::ui::text::{visible_len, char_width};
    
    pub mod widgets {
        pub use crate::ui::textbox::TextBox;
        pub use crate::ui::scrollbar::AccentScrollbar as ScrollBar;
        pub use crate::ui::tabs::AccentTabs as Tabs;
        pub use crate::ui::title_banner::{draw_title_banner, ButtonRect};
        pub use crate::ui::mouse_selection::MouseSelection;
        pub use crate::ui::layout_guard::{is_too_small, render_too_small_warning};
    }

    pub mod text {
        pub use crate::ui::text::*;
    }

    pub mod theme {
        pub use crate::ui::theme::*;
    }

    pub mod layout {
        pub use crate::ui::layout::*;
    }

    pub mod markdown {
        pub use crate::ui::markdown::*;
    }

    pub mod design {
        pub use crate::ui::theme::{ThemeColors, get_theme, accent_color_from_hex};
        pub use crate::ui::layout::centered_rect;
        pub mod prelude {
            pub use crate::ui::layout::{centered_rect, format_help_row};
            pub use crate::ui::text::{wrap_text, visible_len, char_width};
            pub use crate::ui::theme::{ThemeColors, get_theme, accent_color_from_hex};
            pub use crate::ui::markdown::parse_markdown_to_lines;
            pub use crate::ui::status_bar::StatusBar;
            pub use crate::ui::title_banner::ButtonRect;
        }
    }
}


#[cfg(feature = "gui")]
pub mod gui {
    pub use crate::ui::egui_helpers;
    pub use crate::ui::gui_native as native;
}

#[cfg(feature = "interface-api")]
pub mod api {
    pub use crate::toolkit::ipc::{IpcRequest, IpcResponse, IpcServiceHost};
    pub use crate::toolkit::ipc_messages::*;
}

pub mod cli {
}
