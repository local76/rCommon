use crate::sys_info::{PowerStatus, SystemBiosInfo, DiskDriveInfo};

pub fn get_system_screen_resolution() -> (i32, i32) {
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path().join("modes");
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Some(line) = content.lines().next() {
                        let parts: Vec<&str> = line.split('x').collect();
                        if parts.len() == 2 {
                            if let (Ok(w), Ok(h)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                                return (w, h);
                            }
                        }
                    }
                }
            }
        }
    }
    (1920, 1080)
}

pub fn query_os_version() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let val = line.split('=').nth(1).unwrap_or("").trim_matches('"');
                if !val.is_empty() {
                    return val.to_string();
                }
            }
        }
    }
    "Linux".to_string()
}

pub fn query_dark_mode() -> bool {
    // Check Gnome color scheme
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(&["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if s.contains("prefer-dark") {
            return true;
        }
    }
    // Check fallback gtk theme
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
    {
        let s = String::from_utf8_lossy(&output.stdout).to_lowercase();
        if s.contains("dark") {
            return true;
        }
    }
    true
}

pub fn query_power_status() -> Option<PowerStatus> {
    let mut ac_online = true;
    if let Ok(online_str) = std::fs::read_to_string("/sys/class/power_supply/AC/online") {
        ac_online = online_str.trim() == "1";
    } else if let Ok(online_str) = std::fs::read_to_string("/sys/class/power_supply/ADP1/online") {
        ac_online = online_str.trim() == "1";
    }

    let mut battery_percent = 255;
    for bat in &["BAT0", "BAT1"] {
        let path = format!("/sys/class/power_supply/{}/capacity", bat);
        if let Ok(cap_str) = std::fs::read_to_string(path) {
            if let Ok(pct) = cap_str.trim().parse::<u8>() {
                battery_percent = pct;
                break;
            }
        }
    }

    if battery_percent == 255 {
        None
    } else {
        Some(PowerStatus { ac_online, battery_percent })
    }
}

pub fn query_bios_info() -> Option<SystemBiosInfo> {
    let manufacturer = std::fs::read_to_string("/sys/class/dmi/id/sys_vendor")
        .unwrap_or_else(|_| "Linux".to_string())
        .trim()
        .to_string();
    let product = std::fs::read_to_string("/sys/class/dmi/id/product_name")
        .unwrap_or_else(|_| "Generic".to_string())
        .trim()
        .to_string();
    let model = std::fs::read_to_string("/sys/class/dmi/id/product_version")
        .unwrap_or_else(|_| "Kernel".to_string())
        .trim()
        .to_string();
    Some(SystemBiosInfo { manufacturer, product, model })
}

pub fn query_shell_and_terminal() -> (String, String) {
    let shell = std::env::var("SHELL")
        .ok()
        .and_then(|path| {
            std::path::Path::new(&path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "sh".to_string());

    let mut terminal = std::env::var("TERM_PROGRAM").unwrap_or_default();
    if terminal.is_empty() {
        terminal = std::env::var("TERM").unwrap_or_else(|_| "xterm".to_string());
    }
    if terminal.contains("kitty") {
        terminal = "Kitty".to_string();
    } else if terminal.contains("alacritty") {
        terminal = "Alacritty".to_string();
    } else if terminal.contains("wezterm") {
        terminal = "WezTerm".to_string();
    } else if terminal.contains("gnome-terminal") {
        terminal = "GNOME Terminal".to_string();
    } else if terminal.contains("xterm") {
        terminal = "xterm".to_string();
    }

    (shell, terminal)
}

pub fn query_disk_drives() -> Vec<DiskDriveInfo> {
    let mut drives = Vec::new();
    if let Ok(output) = std::process::Command::new("df").args(&["-k"]).output() {
        let s = String::from_utf8_lossy(&output.stdout);
        for line in s.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let path = parts[5].to_string();
                let dev = parts[0];
                if dev.starts_with("/dev/") || path == "/" {
                    if let (Ok(total_kb), Ok(available_kb)) = (parts[1].parse::<u64>(), parts[3].parse::<u64>()) {
                        drives.push(DiskDriveInfo {
                            path,
                            total_bytes: total_kb * 1024,
                            free_bytes: available_kb * 1024,
                        });
                    }
                }
            }
        }
    }
    if drives.is_empty() {
        drives.push(DiskDriveInfo {
            path: "/".to_string(),
            free_bytes: 50 * 1024 * 1024 * 1024,
            total_bytes: 100 * 1024 * 1024 * 1024,
        });
    }
    drives
}
