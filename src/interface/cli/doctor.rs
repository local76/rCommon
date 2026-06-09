//! Diagnostic Doctor and System Checks.
//!
//! Part of the Interface (Presentation Layer) / Role (Application Layer).
//! Verification routines for validating terminal environments, config storage,
//! monitor enumeration, and openrgb daemon connectivity.
//!
//! **Taxonomy Classification**: Interface (CLI) + Role (Application).

/// Doctor/diagnostics helper (generalized from template/helm/trance).
/// Classification: Interface (CLI) + Role (Application).
pub fn run_doctor() {
    run_doctor_with_custom(|_| {});
}

/// Doctor/diagnostics helper that accepts application-specific custom diagnostic checks.
/// Classification: Interface (CLI) + Role (Application).
pub fn run_doctor_with_custom<F>(custom_checks: F)
where
    F: FnOnce(&crate::sys_info::GlyphMap),
{
    println!("===================================================");
    println!("             apps Diagnostic Doctor               ");
    println!("===================================================\n");

    let glyphs = crate::sys_info::GlyphMap::load();

    // 1. Check OS and Local IP
    let os = crate::sys_info::query_os_version();
    println!("{} OS: {}", glyphs.info, os);
    if let Some(ip) = crate::sys_info::query_local_ip() {
        println!("{} Local IP: {}", glyphs.status_ok, ip);
    } else {
        println!("{} Local IP: [Unknown/Offline]", glyphs.warning);
    }

    // 2. Check Registry / Config Persistence
    println!("\nChecking Configuration Storage Access...");
    #[cfg(all(target_os = "windows", feature = "reg"))]
    {
        let test_path = "Software\\apps\\Diagnostics";
        let test_key = "WriteTest";
        let test_val = "ok";
        let write_res = crate::platform::native::reg::write_string(
            crate::platform::native::reg::HKEY_CURRENT_USER,
            test_path,
            test_key,
            test_val,
        );
        if write_res.is_ok() {
            let read_val = crate::platform::native::reg::read_string(
                crate::platform::native::reg::HKEY_CURRENT_USER,
                test_path,
                test_key,
            );
            if read_val == Some(test_val.to_string()) {
                let _ = crate::platform::native::reg::delete_value(
                    crate::platform::native::reg::HKEY_CURRENT_USER,
                    test_path,
                    test_key,
                );
                println!("{} Config Read/Write: Success", glyphs.status_ok);
            } else {
                println!("{} Config Read/Write: Failed (Value mismatch)", glyphs.status_err);
            }
        } else {
            println!("{} Config Read/Write: Failed ({:?})", glyphs.status_err, write_res.err().unwrap());
        }
    }
    #[cfg(not(all(target_os = "windows", feature = "reg")))]
    {
        println!("{} Config Storage: [Skipped - Non-Windows or feature disabled]", glyphs.info);
    }

    // 3. Check Monitor Enumeration
    println!("\nEnumerating Displays...");
    let monitors = crate::platform::native::monitors::get_all_monitors();
    for m in &monitors {
        println!("{} Display: {}", glyphs.info, m);
    }

    // 4. Check OpenRGB Connection
    println!("\nChecking OpenRGB Server Connection...");
    const OPENRGB_PORT: u16 = 6742;
    let rgb_addr = std::net::SocketAddr::from(([127, 0, 0, 1], OPENRGB_PORT));
    match std::net::TcpStream::connect_timeout(&rgb_addr, std::time::Duration::from_millis(500)) {
        Ok(_) => {
            println!("{} OpenRGB Connection: Online (Port {})", glyphs.status_ok, OPENRGB_PORT);
        }
        Err(e) => {
            println!("{} OpenRGB Connection: Offline ({})", glyphs.warning, e);
            println!("   (Is the OpenRGB server running on localhost:{}?)", OPENRGB_PORT);
        }
    }

    // 4.5 Check WinGet SQLite Database
    println!("\nChecking WinGet SQLite Database...");
    #[cfg(all(target_os = "windows", feature = "winget"))]
    {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let db_path = std::path::Path::new(&local_appdata)
                .join("Packages")
                .join("Microsoft.DesktopAppInstaller_8wekyb3d8bbwe")
                .join("LocalState")
                .join("Microsoft.Winget.Source_8wekyb3d8bbwe")
                .join("installed.db");

            if db_path.exists() {
                use rusqlite::{Connection, OpenFlags};
                match Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
                    Ok(conn) => {
                        match conn.query_row("SELECT COUNT(*) FROM manifest", [], |row| {
                            row.get::<_, usize>(0)
                        }) {
                            Ok(count) => {
                                println!("{} WinGet DB: Readable ({} packages, Path: {:?})", glyphs.status_ok, count, db_path);
                            }
                            Err(e) => {
                                println!("{} WinGet DB: Query failed (Error: {}, Path: {:?})", glyphs.status_err, e, db_path);
                            }
                        }
                    }
                    Err(e) => {
                        println!("{} WinGet DB: Open failed (Error: {}, Path: {:?})", glyphs.status_err, e, db_path);
                    }
                }
            } else {
                println!("{} WinGet DB: Not found (Path: {:?})", glyphs.warning, db_path);
            }
        } else {
            println!("{} WinGet DB: Failed (LOCALAPPDATA environment variable not set)", glyphs.status_err);
        }
    }
    #[cfg(not(all(target_os = "windows", feature = "winget")))]
    {
        println!("{} WinGet DB: [Skipped - Non-Windows or feature disabled]", glyphs.info);
    }

    // 5. Run Custom Checks
    custom_checks(&glyphs);

    println!("\n===================================================");
    println!("Diagnostics Complete.");
    println!("===================================================");
}
