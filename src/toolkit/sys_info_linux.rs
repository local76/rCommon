use crate::toolkit::platform::{PowerStatus, SystemBiosInfo, DiskDriveInfo};

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
    let mut has_ac = false;
    let mut battery_percent = None;

    if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(ty_str) = std::fs::read_to_string(path.join("type")) {
                let ty = ty_str.trim();
                if ty == "Mains" {
                    if let Ok(online_str) = std::fs::read_to_string(path.join("online")) {
                        let online = online_str.trim() == "1";
                        if !has_ac {
                            ac_online = online;
                            has_ac = true;
                        } else {
                            ac_online = ac_online || online;
                        }
                    }
                } else if ty == "Battery" {
                    if let Ok(cap_str) = std::fs::read_to_string(path.join("capacity")) {
                        if let Ok(pct) = cap_str.trim().parse::<u8>() {
                            battery_percent = Some(pct);
                        }
                    }
                }
            }
        }
    }

    battery_percent.map(|pct| PowerStatus {
        ac_online: if has_ac { ac_online } else { true },
        battery_percent: pct,
    })
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
    let term_lower = terminal.to_lowercase();
    if term_lower.contains("kitty") {
        terminal = "Kitty".to_string();
    } else if term_lower.contains("alacritty") {
        terminal = "Alacritty".to_string();
    } else if term_lower.contains("wezterm") {
        terminal = "WezTerm".to_string();
    } else if term_lower.contains("gnome-terminal") {
        terminal = "GNOME Terminal".to_string();
    } else if term_lower.contains("xterm") {
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

pub fn query_gpu_names() -> Vec<String> {
    let mut gpus = Vec::new();
    if let Ok(output) = std::process::Command::new("lspci").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("VGA compatible controller") || line.contains("3D controller") {
                if let Some(idx) = line.find(':') {
                    let desc = line[idx + 1..].trim().to_string();
                    if desc.find('[').is_some() {
                        gpus.push(desc);
                    } else {
                        let parts: Vec<&str> = desc.split("controller:").collect();
                        if parts.len() > 1 {
                            gpus.push(parts[1].trim().to_string());
                        } else {
                            gpus.push(desc);
                        }
                    }
                }
            }
        }
    }
    if gpus.is_empty() {
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path().join("device").join("uevent");
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        for line in content.lines() {
                            if line.starts_with("DRIVER=") {
                                let driver = line.split('=').nth(1).unwrap_or("").to_string();
                                if !driver.is_empty() && !gpus.contains(&driver) {
                                    gpus.push(driver);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    gpus
}

pub fn query_network_adapters() -> Vec<crate::platform::NetworkAdapterInfo> {
    use crate::platform::NetworkAdapterInfo;
    let mut adapters: Vec<NetworkAdapterInfo> = Vec::new();

    // Use "ip -o addr" for IPs + interfaces (common on Linux).
    if let Ok(output) = std::process::Command::new("ip").args(&["-o", "addr", "show"]).output() {
        let s = String::from_utf8_lossy(&output.stdout);
        for line in s.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let iface = parts[1].to_string();
                let ip = parts[3].split('/').next().unwrap_or("").to_string();
                if ip.is_empty() { continue; }

                let mut adapter_type = "Other".to_string();
                let lower = iface.to_lowercase();
                if lower.contains("wlan") || lower.contains("wifi") || lower.contains("wl") {
                    adapter_type = "Wi-Fi".to_string();
                } else if lower.contains("bt") || lower.contains("bluetooth") {
                    adapter_type = "Bluetooth".to_string();
                } else if lower.contains("eth") || lower.contains("enp") {
                    adapter_type = "Ethernet".to_string();
                } else if lower.contains("lo") {
                    adapter_type = "Loopback".to_string();
                }

                // merge IPs for same iface
                if let Some(existing) = adapters.iter_mut().find(|a| a.name == iface) {
                    if !existing.ip_addresses.contains(&ip) {
                        existing.ip_addresses.push(ip);
                    }
                } else {
                    adapters.push(NetworkAdapterInfo {
                        name: iface,
                        description: "Linux interface".to_string(),
                        ip_addresses: vec![ip],
                        adapter_type,
                        is_up: true,
                    });
                }
            }
        }
    }

    if adapters.is_empty() {
        adapters.push(NetworkAdapterInfo {
            name: "lo".to_string(),
            description: "Loopback".to_string(),
            ip_addresses: vec!["127.0.0.1".to_string()],
            adapter_type: "Loopback".to_string(),
            is_up: true,
        });
    }
    adapters
}

