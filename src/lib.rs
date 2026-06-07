#[cfg(feature = "clipboard")]
pub mod clipboard;
#[cfg(feature = "service")]
pub mod service;
#[cfg(feature = "event-log")]
pub mod event_log;
#[cfg(feature = "notification")]
pub mod notification;
#[cfg(feature = "window")]
pub mod window;
#[cfg(feature = "window")]
pub mod guard;

#[cfg(feature = "sys-info")]
pub mod sys_info;

#[cfg(feature = "reg")]
pub mod reg;

#[cfg(feature = "widgets")]
pub mod widgets;

#[cfg(feature = "rgb")]
pub mod rgb;

// Deprecated backwards-compatibility module.
#[cfg(feature = "win32")]
#[deprecated(since = "1.0.0", note = "Use the domain-specific modules instead")]
pub mod win32 {
    #[cfg(feature = "clipboard")]
    pub use crate::clipboard::copy_text_to_clipboard;
    
    #[cfg(feature = "service")]
    pub use crate::service::{query_service_status as query_windows_service_status, SERVICE_STATUS};
    
    #[cfg(feature = "event-log")]
    pub use crate::event_log::log_system_event as log_windows_event;
    
    #[cfg(feature = "notification")]
    pub use crate::notification::show_toast_notification;
    
    #[cfg(feature = "window")]
    pub use crate::window::{
        center_console_window, get_console_rect, get_window_rect, query_cursor_pos,
        relaunch_in_conhost_if_needed, set_window_pos, BorderlessConsole, ConsoleTitleGuard,
        SingleInstanceGuard, COORD, CONSOLE_SELECTION_INFO, MONITORINFO, POINT, RECT, SMALL_RECT,
    };
    
    #[cfg(feature = "sys-info")]
    pub use crate::sys_info::{
        get_console_window_dpi, get_system_screen_resolution,
        query_bios_info, query_dark_mode, query_disk_drives, query_local_ip, query_os_version,
        query_power_status, query_shell_and_terminal, DiskDriveInfo, GlyphMap, PowerStatus,
        SystemBiosInfo,
    };
    
    #[cfg(all(feature = "sys-info", feature = "widgets"))]
    pub use crate::sys_info::get_dwm_accent_color;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_ip() {
        #[cfg(feature = "sys-info")]
        {
            let ip = sys_info::query_local_ip();
            println!("Local IP: {:?}", ip);
        }
    }

    #[test]
    fn test_xml_escaping() {
        #[cfg(feature = "notification")]
        notification::show_toast_notification("<test>&", "\"message'\"");
    }

    #[test]
    fn test_registry_persistence() {
        #[cfg(feature = "reg")]
        {
            let key_name = "test_config_key";
            let path = "Software\\rApps\\Test";
            
            // 1. Initial read should be None
            let _ = reg::delete_value(reg::HKEY_CURRENT_USER, path, key_name);
            let val_init = reg::read_string(reg::HKEY_CURRENT_USER, path, key_name);
            assert_eq!(val_init, None);

            // 2. Write key
            let write_ok = reg::write_string(reg::HKEY_CURRENT_USER, path, key_name, "hello_world");
            if let Err(ref e) = write_ok {
                panic!("Write failed with error: {:?}", e);
            }
            assert!(write_ok.is_ok());

            // 3. Read back
            let val = reg::read_string(reg::HKEY_CURRENT_USER, path, key_name);
            assert_eq!(val, Some("hello_world".to_string()));

            // 4. Delete key
            let delete_ok = reg::delete_value(reg::HKEY_CURRENT_USER, path, key_name);
            assert!(delete_ok.is_ok());

            // 5. Read back again
            let val_post = reg::read_string(reg::HKEY_CURRENT_USER, path, key_name);
            assert_eq!(val_post, None);
        }
    }

    #[test]
    fn test_sys_info_stubs() {
        #[cfg(feature = "sys-info")]
        {
            let res = sys_info::get_system_screen_resolution();
            assert!(res.0 > 0 && res.1 > 0);
            let dpi = sys_info::get_console_window_dpi();
            assert!(dpi > 0);
        }
    }
}
