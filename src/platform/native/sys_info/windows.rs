use crate::platform::{PowerStatus, SystemBiosInfo, DiskDriveInfo};
#[cfg(feature = "widgets")]
use ratatui::style::Color;

pub fn query_accent_color() -> (u8, u8, u8) {
    let mut color: u32 = 0;
    let mut opaque: i32 = 0;
    let hr = unsafe {
        windows_sys::Win32::Graphics::Dwm::DwmGetColorizationColor(&mut color, &mut opaque)
    };
    if hr == 0 {
        let r = ((color >> 16) & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8;
        let b = (color & 0xFF) as u8;
        (r, g, b)
    } else {
        (0, 245, 255)
    }
}

#[cfg(feature = "widgets")]
pub fn get_dwm_accent_color() -> Color {
    let (r, g, b) = query_accent_color();
    Color::Rgb(r, g, b)
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
    sys.refresh_processes_specifics(sysinfo::ProcessRefreshKind::new());

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

pub fn query_gpu_names() -> Vec<String> {
    let mut gpus = Vec::new();
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
    if let Ok(class_key) = hklm.open_subkey(path) {
        for subkey_name in class_key.enum_keys().filter_map(|x| x.ok()) {
            if subkey_name.len() == 4 && subkey_name.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(gpu_key) = class_key.open_subkey(&subkey_name) {
                    if let Ok(desc) = gpu_key.get_value::<String, _>("DriverDesc") {
                        gpus.push(desc);
                    }
                }
            }
        }
    }
    gpus
}

pub fn query_network_adapters() -> Vec<crate::platform::NetworkAdapterInfo> {
    use crate::platform::NetworkAdapterInfo;
    let mut adapters = Vec::new();

    // Pragmatic but effective: use ipconfig /all (widely available, no extra FFI complexity for sockaddr parsing).
    // Detects Wi-Fi, Bluetooth, Ethernet adapters + their IPv4/IPv6 addresses.
    if let Ok(output) = std::process::Command::new("ipconfig").arg("/all").output() {
        let s = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = s.lines().collect();
        let mut current_name = String::new();
        let mut current_desc = String::new();
        let mut current_ips: Vec<String> = vec![];
        let mut current_type = "Other".to_string();
        let mut is_up = true;

        for line in lines {
            let trimmed = line.trim_start();
            if trimmed.ends_with(':') && !trimmed.starts_with(' ') && (trimmed.contains("adapter") || trimmed.contains("Adapter")) {
                if !current_name.is_empty() {
                    adapters.push(NetworkAdapterInfo {
                        name: current_name.clone(),
                        description: current_desc.clone(),
                        ip_addresses: current_ips.clone(),
                        adapter_type: current_type.clone(),
                        is_up,
                    });
                }
                current_name = trimmed.trim_end_matches(':').to_string();
                current_desc.clear();
                current_ips.clear();
                let lower = current_name.to_lowercase();
                current_type = if lower.contains("wi-fi") || lower.contains("wireless") || lower.contains("wlan") {
                    "Wi-Fi".to_string()
                } else if lower.contains("bluetooth") {
                    "Bluetooth".to_string()
                } else if lower.contains("ethernet") || lower.contains("local area") {
                    "Ethernet".to_string()
                } else if lower.contains("virtual") || lower.contains("loopback") || lower.contains("vethernet") {
                    "Virtual".to_string()
                } else {
                    "Other".to_string()
                };
                is_up = true;
            } else if trimmed.starts_with("Description") {
                if let Some(val) = trimmed.split_once('.').map(|x| x.1) {
                    current_desc = val.trim().to_string();
                }
            } else if trimmed.starts_with("IPv4 Address") || trimmed.starts_with("IPv6 Address") || trimmed.contains("Autoconfiguration IPv4 Address") || trimmed.contains("Link-local IPv6 Address") {
                if let Some(val) = trimmed.split_once(':').map(|x| x.1) {
                    let ip = val.trim().trim_end_matches("(Preferred)").trim().to_string();
                    if !ip.is_empty() && !current_ips.contains(&ip) {
                        current_ips.push(ip);
                    }
                }
            } else if trimmed.to_lowercase().contains("media disconnected") {
                is_up = false;
            }
        }
        if !current_name.is_empty() {
            adapters.push(NetworkAdapterInfo {
                name: current_name,
                description: current_desc,
                ip_addresses: current_ips,
                adapter_type: current_type,
                is_up,
            });
        }
    }

    if adapters.is_empty() {
        adapters.push(NetworkAdapterInfo {
            name: "Unknown Adapter".to_string(),
            description: "ipconfig parse failed".to_string(),
            ip_addresses: vec!["127.0.0.1".to_string()],
            adapter_type: "Other".to_string(),
            is_up: true,
        });
    }
    adapters
}

pub fn query_high_contrast() -> bool {
    use windows_sys::Win32::UI::Accessibility::{HIGHCONTRASTW, HCF_HIGHCONTRASTON};
    use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETHIGHCONTRAST};

    let mut hc: HIGHCONTRASTW = unsafe { std::mem::zeroed() };
    hc.cbSize = std::mem::size_of::<HIGHCONTRASTW>() as u32;
    let res = unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            hc.cbSize,
            &mut hc as *mut _ as *mut _,
            0,
        )
    };
    if res == 0 {
        return false;
    }
    (hc.dwFlags & HCF_HIGHCONTRASTON) != 0
}

