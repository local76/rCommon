//! Package / software inventory utilities.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).
//! 
//! Ported and generalized from rFetch (worker_win.rs) and similar patterns in rMonitor/rStartup.
//! Provides counts for common package managers and a breakdown string for dashboards/TUIs.
//! Full detailed scanning lives in app-specific code (e.g. rFetch for async), but these are reusable helpers.
//!
//! For taxonomy details, see [ARCHITECTURE.md](file:///C:/Users/jeryd/Synology/Home/Projects/local76/rCommon/ARCHITECTURE.md).
//! Cross-platform with native features and platform-specific stubs.

use std::fs;
use std::path::Path;

#[cfg(all(windows, feature = "reg"))]
use winreg::RegKey;
#[cfg(all(windows, feature = "reg"))]
use winreg::enums::*;

pub fn count_scoop() -> usize {
    let scoop_dir = std::env::var("SCOOP")
        .map(|s| Path::new(&s).join("apps"))
        .ok()
        .or_else(|| {
            std::env::var("USERPROFILE")
                .map(|home| Path::new(&home).join("scoop").join("apps"))
                .ok()
        });

    if let Some(path) = scoop_dir {
        if path.exists() {
            if let Ok(entries) = fs::read_dir(path) {
                return entries.count();
            }
        }
    }
    0
}

pub fn count_choco() -> usize {
    let choco_dir = std::env::var("ChocolateyInstall")
        .map(|s| Path::new(&s).join("lib"))
        .ok()
        .unwrap_or_else(|| Path::new(r"C:\ProgramData\chocolatey\lib").to_path_buf());

    if choco_dir.exists() {
        if let Ok(entries) = fs::read_dir(choco_dir) {
            let total = entries.count();
            return total.saturating_sub(1);
        }
    }
    0
}

pub fn count_npm() -> usize {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let npm_dir = Path::new(&appdata)
            .join("npm")
            .join("node_modules");
        if npm_dir.exists() {
            if let Ok(entries) = fs::read_dir(npm_dir) {
                return entries.count();
            }
        }
    }
    0
}

pub fn count_steam() -> usize {
    #[allow(unused_mut)]
    let mut count = 0;
    #[cfg(all(windows, feature = "reg"))]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(apps_key) = hkcu.open_subkey_with_flags(r"Software\Valve\Steam\Apps", KEY_READ) {
            for name in apps_key.enum_keys().filter_map(|x| x.ok()) {
                if let Ok(game_key) = apps_key.open_subkey_with_flags(&name, KEY_READ) {
                    if let Ok(val) = game_key.get_value::<u32, _>("Installed") {
                        if val == 1 {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let mut paths = Vec::new();
        if let Ok(home) = std::env::var("HOME") {
            let p = Path::new(&home);
            paths.push(p.join(".steam").join("steam").join("steamapps"));
            paths.push(p.join(".local").join("share").join("Steam").join("steamapps"));
            paths.push(p.join(".var").join("app").join("com.valvesoftware.Steam").join(".local").join("share").join("Steam").join("steamapps"));
        }
        for path in paths {
            if path.exists() {
                if let Ok(entries) = fs::read_dir(path) {
                    let mut found_manifests = 0;
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.starts_with("appmanifest_") && name.ends_with(".acf") {
                                found_manifests += 1;
                            }
                        }
                    }
                    if found_manifests > 0 {
                        count = found_manifests;
                        break;
                    }
                }
            }
        }
    }
    count
}

pub fn count_ms_store() -> usize {
    #[allow(unused_mut)]
    let mut count = 0;
    #[cfg(all(windows, feature = "reg"))]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppModel\Repository\Packages";
        if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_READ) {
            count = key.enum_keys().count();
        }
    }
    count
}

pub fn count_native() -> usize {
    #[allow(unused_mut)]
    let mut count = 0;
    #[cfg(all(windows, feature = "reg"))]
    {
        let locations = [
            (
                HKEY_CURRENT_USER,
                r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                HKEY_LOCAL_MACHINE,
                r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                HKEY_LOCAL_MACHINE,
                r"Software\Wow6432Node\Software\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
        ];
        for &(hkey, path) in &locations {
            let root = RegKey::predef(hkey);
            if let Ok(key) = root.open_subkey_with_flags(path, KEY_READ) {
                count += key.enum_keys().count();
            }
        }
    }
    count
}

pub fn count_winget() -> usize {
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let db_path = Path::new(&local_appdata)
            .join("Packages")
            .join("Microsoft.DesktopAppInstaller_8wekyb3d8bbwe")
            .join("LocalState")
            .join("Microsoft.Winget.Source_8wekyb3d8bbwe")
            .join("installed.db");

        if db_path.exists() {
            // rusqlite is used in rFetch for this; optional here to avoid extra dep in rCommon core.
            // Full impl can be enabled via feature or left to apps.
            // **Feature Stub**: This is a fallback placeholder implementation designed to compile successfully and preserve API parity.
            // if db_path.exists() { ... full query ... }
            // For now returns 0; apps can extend.
        }
    }
    0
}

pub fn count_dpkg() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = fs::read_dir("/var/lib/dpkg/info") {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "list"))
                .count()
        } else {
            0
        }
    }
    #[cfg(not(target_os = "linux"))]
    0
}

pub fn count_pacman() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = fs::read_dir("/var/lib/pacman/local") {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .count()
        } else {
            0
        }
    }
    #[cfg(not(target_os = "linux"))]
    0
}

pub fn count_flatpak() -> usize {
    #[cfg(target_os = "linux")]
    {
        let mut count = 0;
        for base in &["/var/lib/flatpak/app", "/var/lib/flatpak/runtime"] {
            if let Ok(entries) = fs::read_dir(base) {
                count += entries.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count();
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            for base in &["share/flatpak/app", "share/flatpak/runtime"] {
                let p = Path::new(&home).join(".local").join(base);
                if let Ok(entries) = fs::read_dir(p) {
                    count += entries.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count();
                }
            }
        }
        count
    }
    #[cfg(not(target_os = "linux"))]
    0
}

pub fn count_snap() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = fs::read_dir("/var/lib/snapd/snaps") {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "snap"))
                .count()
        } else {
            0
        }
    }
    #[cfg(not(target_os = "linux"))]
    0
}

/// Information about a package manager supported by rCommon.
#[derive(Debug, Clone, Copy)]
pub struct PackageManager {
    /// The name of the package manager (e.g. "scoop", "npm", "dpkg").
    pub name: &'static str,
    /// Function to count the number of installed packages for this manager.
    pub count_fn: fn() -> usize,
}

/// Global registry of package managers supported for local scanning.
pub static PACKAGE_MANAGERS: &[PackageManager] = &[
    PackageManager { name: "native", count_fn: count_native },
    PackageManager { name: "winget", count_fn: count_winget },
    PackageManager { name: "ms-store", count_fn: count_ms_store },
    PackageManager { name: "scoop", count_fn: count_scoop },
    PackageManager { name: "choco", count_fn: count_choco },
    PackageManager { name: "steam", count_fn: count_steam },
    PackageManager { name: "npm", count_fn: count_npm },
    PackageManager { name: "dpkg", count_fn: count_dpkg },
    PackageManager { name: "pacman", count_fn: count_pacman },
    PackageManager { name: "flatpak", count_fn: count_flatpak },
    PackageManager { name: "snap", count_fn: count_snap },
];

/// Returns a human-readable breakdown of installed packages.
/// Classification: Role (Application) + Platform (Native).
/// Ported from rFetch. Useful for TUIs, CLIs, and dashboards.
pub fn get_packages_breakdown() -> String {
    static CACHE: std::sync::Mutex<Option<(std::time::Instant, String)>> = std::sync::Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(5000) {
            return val.clone();
        }
    }
    let val = get_packages_breakdown_uncached();
    *lock = Some((std::time::Instant::now(), val.clone()));
    val
}

fn get_packages_breakdown_uncached() -> String {
    let mut parts = Vec::new();
    for manager in PACKAGE_MANAGERS {
        let count = (manager.count_fn)();
        if count > 0 {
            parts.push(format!("{} {}", count, manager.name));
        }
    }

    if parts.is_empty() {
        "0 apps".to_string()
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_native_fallback() {
        // On non-Windows, returns 0.
        #[cfg(not(windows))]
        assert_eq!(count_native(), 0);
    }

    #[test]
    fn test_packages_breakdown() {
        let breakdown = get_packages_breakdown();
        assert!(breakdown.contains("apps") || breakdown.contains("native"));
    }
}