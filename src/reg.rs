#![allow(dead_code, unused_imports, unused_variables)]
use std::io;

#[cfg(all(feature = "reg", target_os = "windows"))]
use winreg::RegKey;
#[cfg(all(feature = "reg", target_os = "windows"))]
use winreg::enums::KEY_READ;

#[cfg(all(feature = "reg", target_os = "windows"))]
pub use winreg::enums::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS};
#[cfg(all(feature = "reg", target_os = "windows"))]
pub use winreg::HKEY;

#[cfg(not(all(feature = "reg", target_os = "windows")))]
pub type HKEY = isize;
#[cfg(not(all(feature = "reg", target_os = "windows")))]
pub const HKEY_CLASSES_ROOT: HKEY = 0;
#[cfg(not(all(feature = "reg", target_os = "windows")))]
pub const HKEY_CURRENT_USER: HKEY = 1;
#[cfg(not(all(feature = "reg", target_os = "windows")))]
pub const HKEY_LOCAL_MACHINE: HKEY = 2;
#[cfg(not(all(feature = "reg", target_os = "windows")))]
pub const HKEY_USERS: HKEY = 3;

#[cfg(not(all(feature = "reg", target_os = "windows")))]
fn get_registry_file_path() -> Option<std::path::PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        Some(std::path::PathBuf::from(xdg).join("rApps").join("registry.conf"))
    } else if let Ok(home) = std::env::var("HOME") {
        Some(std::path::PathBuf::from(home).join(".config").join("rApps").join("registry.conf"))
    } else {
        None
    }
}

#[cfg(not(all(feature = "reg", target_os = "windows")))]
fn read_entry(hive: HKEY, path: &str, key: &str) -> Option<(char, String)> {
    let file_path = get_registry_file_path()?;
    if !file_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(file_path).ok()?;
    let prefix = format!("{}::{}::{}=", hive, path.to_lowercase(), key.to_lowercase());
    for line in content.lines() {
        if line.starts_with(&prefix) {
            let val_part = &line[prefix.len()..];
            if val_part.len() >= 2 && val_part.as_bytes()[1] == b':' {
                let vtype = val_part.chars().next().unwrap();
                let value = val_part[2..].to_string();
                return Some((vtype, value));
            }
        }
    }
    None
}

#[cfg(not(all(feature = "reg", target_os = "windows")))]
fn write_entry(hive: HKEY, path: &str, key: &str, vtype: char, val: &str) -> io::Result<()> {
    let file_path = match get_registry_file_path() {
        Some(p) => p,
        None => return Err(io::Error::new(io::ErrorKind::NotFound, "No home directory resolved")),
    };
    if let Some(parent) = file_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    let mut lines = Vec::new();
    if file_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            lines = content.lines().map(|s| s.to_string()).collect();
        }
    }
    
    let prefix = format!("{}::{}::{}=", hive, path.to_lowercase(), key.to_lowercase());
    let new_line = format!("{}{}:{}", prefix, vtype, val);
    
    let mut found = false;
    for line in &mut lines {
        if line.starts_with(&prefix) {
            *line = new_line.clone();
            found = true;
            break;
        }
    }
    if !found {
        lines.push(new_line);
    }
    
    std::fs::write(&file_path, lines.join("\n"))?;
    Ok(())
}

#[cfg(not(all(feature = "reg", target_os = "windows")))]
fn delete_entry(hive: HKEY, path: &str, key: &str) -> io::Result<()> {
    let file_path = match get_registry_file_path() {
        Some(p) => p,
        None => return Err(io::Error::new(io::ErrorKind::NotFound, "No home directory resolved")),
    };
    if !file_path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&file_path)?;
    let prefix = format!("{}::{}::{}=", hive, path.to_lowercase(), key.to_lowercase());
    let mut lines = Vec::new();
    for line in content.lines() {
        if !line.starts_with(&prefix) {
            lines.push(line.to_string());
        }
    }
    std::fs::write(&file_path, lines.join("\n"))?;
    Ok(())
}

/// Read a string value from the registry.
pub fn read_string(hive: HKEY, path: &str, key: &str) -> Option<String> {
    #[cfg(all(feature = "reg", target_os = "windows"))]
    {
        let root = RegKey::predef(hive);
        let subkey = root.open_subkey_with_flags(path, KEY_READ).ok()?;
        subkey.get_value::<String, _>(key).ok()
    }
    #[cfg(not(all(feature = "reg", target_os = "windows")))]
    {
        read_entry(hive, path, key).and_then(|(t, v)| if t == 'S' { Some(v) } else { None })
    }
}

/// Write a string value to the registry.
pub fn write_string(hive: HKEY, path: &str, key: &str, val: &str) -> io::Result<()> {
    #[cfg(all(feature = "reg", target_os = "windows"))]
    {
        let root = RegKey::predef(hive);
        let (subkey, _) = root.create_subkey(path)?;
        subkey.set_value(key, &val.to_string())
    }
    #[cfg(not(all(feature = "reg", target_os = "windows")))]
    {
        write_entry(hive, path, key, 'S', val)
    }
}

/// Read a u32 (DWORD) value from the registry.
pub fn read_u32(hive: HKEY, path: &str, key: &str) -> Option<u32> {
    #[cfg(all(feature = "reg", target_os = "windows"))]
    {
        let root = RegKey::predef(hive);
        let subkey = root.open_subkey_with_flags(path, KEY_READ).ok()?;
        subkey.get_value::<u32, _>(key).ok()
    }
    #[cfg(not(all(feature = "reg", target_os = "windows")))]
    {
        read_entry(hive, path, key).and_then(|(t, v)| if t == 'D' { v.parse::<u32>().ok() } else { None })
    }
}

/// Write a u32 (DWORD) value to the registry.
pub fn write_u32(hive: HKEY, path: &str, key: &str, val: u32) -> io::Result<()> {
    #[cfg(all(feature = "reg", target_os = "windows"))]
    {
        let root = RegKey::predef(hive);
        let (subkey, _) = root.create_subkey(path)?;
        subkey.set_value(key, &val)
    }
    #[cfg(not(all(feature = "reg", target_os = "windows")))]
    {
        write_entry(hive, path, key, 'D', &val.to_string())
    }
}

/// Read a boolean value represented as "1" or "0" string in the registry.
pub fn read_bool_str(hive: HKEY, path: &str, key: &str) -> bool {
    read_string(hive, path, key)
        .map(|s| s.trim() == "1")
        .unwrap_or(false)
}

/// Write a boolean value as a "1" or "0" string to the registry.
pub fn write_bool_str(hive: HKEY, path: &str, key: &str, val: bool) -> io::Result<()> {
    let str_val = if val { "1" } else { "0" };
    write_string(hive, path, key, str_val)
}

/// List all key-value string pairs in a registry subkey.
pub fn list_values(hive: HKEY, path: &str) -> Option<Vec<(String, String)>> {
    #[cfg(all(feature = "reg", target_os = "windows"))]
    {
        let root = RegKey::predef(hive);
        let subkey = root.open_subkey_with_flags(path, KEY_READ).ok()?;
        let mut values = Vec::new();
        for (name, _) in subkey.enum_values().flatten() {
            if let Ok(val) = subkey.get_value::<String, _>(&name) {
                values.push((name, val));
            }
        }
        Some(values)
    }
    #[cfg(not(all(feature = "reg", target_os = "windows")))]
    {
        let file_path = get_registry_file_path()?;
        if !file_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(file_path).ok()?;
        let prefix = format!("{}::{}::", hive, path.to_lowercase());
        let mut values = Vec::new();
        for line in content.lines() {
            if line.starts_with(&prefix) {
                let rest = &line[prefix.len()..];
                if let Some(idx) = rest.find('=') {
                    let key_name = &rest[..idx];
                    let val_part = &rest[idx+1..];
                    if val_part.len() >= 2 && val_part.as_bytes()[1] == b':' {
                        let vtype = val_part.chars().next().unwrap();
                        let val_str = &val_part[2..];
                        if vtype == 'S' {
                            values.push((key_name.to_string(), val_str.to_string()));
                        }
                    }
                }
            }
        }
        Some(values)
    }
}

/// Read a binary value from the registry.
pub fn read_binary(hive: HKEY, path: &str, key: &str) -> Option<Vec<u8>> {
    #[cfg(all(feature = "reg", target_os = "windows"))]
    {
        let root = RegKey::predef(hive);
        let subkey = root.open_subkey_with_flags(path, KEY_READ).ok()?;
        let value = subkey.get_raw_value(key).ok()?;
        Some(value.bytes.to_vec())
    }
    #[cfg(not(all(feature = "reg", target_os = "windows")))]
    {
        read_entry(hive, path, key).and_then(|(t, v)| {
            if t == 'B' {
                let mut bytes = Vec::new();
                let mut chars = v.chars();
                while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
                    if let Ok(b) = u8::from_str_radix(&format!("{}{}", c1, c2), 16) {
                        bytes.push(b);
                    }
                }
                Some(bytes)
            } else {
                None
            }
        })
    }
}

/// Write a binary value to the registry.
pub fn write_binary(hive: HKEY, path: &str, key: &str, val: &[u8]) -> io::Result<()> {
    #[cfg(all(feature = "reg", target_os = "windows"))]
    {
        let root = RegKey::predef(hive);
        let (subkey, _) = root.create_subkey(path)?;
        let reg_val = winreg::RegValue {
            vtype: winreg::enums::REG_BINARY,
            bytes: std::borrow::Cow::Borrowed(val),
        };
        subkey.set_raw_value(key, &reg_val)
    }
    #[cfg(not(all(feature = "reg", target_os = "windows")))]
    {
        let hex: String = val.iter().map(|b| format!("{:02x}", b)).collect();
        write_entry(hive, path, key, 'B', &hex)
    }
}

/// Delete a value from the registry.
pub fn delete_value(hive: HKEY, path: &str, key: &str) -> io::Result<()> {
    #[cfg(all(feature = "reg", target_os = "windows"))]
    {
        let root = RegKey::predef(hive);
        let subkey = root.open_subkey_with_flags(path, winreg::enums::KEY_WRITE)?;
        subkey.delete_value(key)
    }
    #[cfg(not(all(feature = "reg", target_os = "windows")))]
    {
        delete_entry(hive, path, key)
    }
}
