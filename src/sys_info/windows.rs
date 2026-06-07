use crate::sys_info::{PowerStatus, SystemBiosInfo, DiskDriveInfo};
#[cfg(feature = "widgets")]
use ratatui::style::Color;

#[cfg(feature = "widgets")]
pub fn get_dwm_accent_color() -> Color {
    let mut color: u32 = 0;
    let mut opaque: i32 = 0;
    let hr = unsafe {
        windows_sys::Win32::Graphics::Dwm::DwmGetColorizationColor(&mut color, &mut opaque)
    };
    if hr == 0 {
        let r = ((color >> 16) & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8;
        let b = (color & 0xFF) as u8;
        return Color::Rgb(r, g, b);
    }
    Color::Rgb(0, 245, 255)
}

pub fn get_system_screen_resolution() -> (i32, i32) {
    let screen_w = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
            windows_sys::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
        )
    };
    let screen_h = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
            windows_sys::Win32::UI::WindowsAndMessaging::SM_CYSCREEN,
        )
    };
    (screen_w, screen_h)
}

pub fn get_console_window_dpi() -> u32 {
    let hwnd = unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() };
    if !hwnd.is_null() {
        let dpi = unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForWindow(hwnd) };
        if dpi > 0 { dpi } else { 96 }
    } else {
        96
    }
}

pub fn query_os_version() -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut product_name = "Windows".to_string();
    let mut current_build = String::new();
    let mut display_version = String::new();

    if let Ok(key) = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion") {
        if let Ok(val) = key.get_value::<String, _>("ProductName") {
            product_name = val;
        }
        if let Ok(val) = key.get_value::<String, _>("CurrentBuild") {
            current_build = val;
        }
        if let Ok(val) = key.get_value::<String, _>("DisplayVersion") {
            display_version = val;
        }
    }

    let mut final_product = product_name;
    if final_product.starts_with("Windows 10") {
        if let Ok(build) = current_build.parse::<u32>() {
            if build >= 22000 {
                final_product = final_product.replace("Windows 10", "Windows 11");
            }
        }
    }

    let mut parts = vec![final_product];
    if !display_version.is_empty() {
        parts.push(display_version);
    }
    if !current_build.is_empty() {
        parts.push(format!("(Build {})", current_build));
    }
    parts.join(" ")
}

pub fn query_dark_mode() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize") {
        if let Ok(val) = key.get_value::<u32, _>("AppsUseLightTheme") {
            return val == 0;
        }
    }
    true
}

pub fn query_power_status() -> Option<PowerStatus> {
    let mut status = windows_sys::Win32::System::Power::SYSTEM_POWER_STATUS {
        ACLineStatus: 0,
        BatteryFlag: 0,
        BatteryLifePercent: 0,
        SystemStatusFlag: 0,
        BatteryLifeTime: 0,
        BatteryFullLifeTime: 0,
    };
    if unsafe {
        windows_sys::Win32::System::Power::GetSystemPowerStatus(&mut status)
    } != 0
    {
        return Some(PowerStatus {
            ac_online: status.ACLineStatus == 1,
            battery_percent: status.BatteryLifePercent,
        });
    }
    None
}

pub fn query_bios_info() -> Option<SystemBiosInfo> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = r"HARDWARE\DESCRIPTION\System\BIOS";
    if let Ok(key) = hklm.open_subkey(path) {
        let manufacturer = key
            .get_value::<String, _>("SystemManufacturer")
            .unwrap_or_default();
        let product = key
            .get_value::<String, _>("SystemProductName")
            .unwrap_or_default();
        let model = key
            .get_value::<String, _>("BaseBoardProduct")
            .unwrap_or_default();
        return Some(SystemBiosInfo {
            manufacturer: manufacturer.trim().to_string(),
            product: product.trim().to_string(),
            model: model.trim().to_string(),
        });
    }
    None
}

pub fn query_shell_and_terminal() -> (String, String) {
    let mut shell = "Unknown Shell".to_string();
    let mut terminal = "Unknown Terminal".to_string();

    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_processes();

    let mut current_pid = sysinfo::get_current_pid().ok();
    let mut depth = 0;

    while let Some(pid) = current_pid {
        if depth > 12 {
            break;
        }
        if let Some(process) = sys.process(pid) {
            let name = process.name().to_lowercase();
            if shell == "Unknown Shell" {
                if name.contains("powershell") || name.contains("pwsh") {
                    shell = "PowerShell".to_string();
                } else if name == "cmd.exe" || name == "cmd" {
                    shell = "CMD".to_string();
                } else if name.contains("bash") || name.contains("sh") || name.contains("zsh") {
                    shell = name.replace(".exe", "");
                }
            }

            if terminal == "Unknown Terminal" {
                if name.contains("windowsterminal") || name == "openconsole.exe" {
                    terminal = "Windows Terminal".to_string();
                } else if name.contains("code") {
                    terminal = "VS Code Terminal".to_string();
                } else if name.contains("alacritty") {
                    terminal = "Alacritty".to_string();
                } else if name.contains("wezterm") {
                    terminal = "WezTerm".to_string();
                } else if name.contains("conhost") {
                    terminal = "Windows Console Host".to_string();
                }
            }

            current_pid = process.parent();
            depth += 1;
        } else {
            break;
        }
    }
    (shell, terminal)
}

pub fn query_disk_drives() -> Vec<DiskDriveInfo> {
    let mut drives = Vec::new();
    unsafe {
        let mut buffer = [0u16; 512];
        let len = windows_sys::Win32::Storage::FileSystem::GetLogicalDriveStringsW(
            buffer.len() as u32,
            buffer.as_mut_ptr(),
        );
        if len > 0 && len < buffer.len() as u32 {
            let mut start = 0;
            for idx in 0..len as usize {
                if buffer[idx] == 0 {
                    if idx > start {
                        let drive_w = &buffer[start..idx];
                        let mut path_null = drive_w.to_vec();
                        path_null.push(0);

                        let mut free_caller = 0u64;
                        let mut total = 0u64;
                        let mut free_total = 0u64;
                        let ok = windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                            path_null.as_ptr(),
                            &mut free_caller,
                            &mut total,
                            &mut free_total,
                        );
                        if ok != 0 {
                            if let Ok(path) = String::from_utf16(drive_w) {
                                drives.push(DiskDriveInfo {
                                    path: path.trim_end_matches('\0').to_string(),
                                    total_bytes: total,
                                    free_bytes: free_caller,
                                });
                            }
                        }
                    }
                    start = idx + 1;
                }
            }
        }
    }
    drives
}
