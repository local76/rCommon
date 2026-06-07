use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("=== rpack Centralized Packaging Tool ===");

    // 1. Verify we are in a cargo project directory
    if !Path::new("Cargo.toml").exists() {
        eprintln!("Error: Cargo.toml not found in the current directory.");
        eprintln!("Please run rpack from the root of the application you want to package.");
        std::process::exit(1);
    }

    // 2. Parse name and version from Cargo.toml
    let content = fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    let mut name = None;
    let mut version = None;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name =") && name.is_none() {
            name = Some(
                line.split('=')
                    .nth(1)
                    .unwrap()
                    .trim()
                    .trim_matches('"')
                    .to_string(),
            );
        }
        if line.starts_with("version =") && version.is_none() {
            version = Some(
                line.split('=')
                    .nth(1)
                    .unwrap()
                    .trim()
                    .trim_matches('"')
                    .to_string(),
            );
        }
    }

    let Some(app_name) = name else {
        eprintln!("Error: Could not parse package name from Cargo.toml.");
        std::process::exit(1);
    };
    let Some(app_version) = version else {
        eprintln!("Error: Could not parse package version from Cargo.toml.");
        std::process::exit(1);
    };

    println!("Packaging application: {} (version: {})", app_name, app_version);

    // 3. Create dist directory structure
    let dist_dir = Path::new("dist");
    if dist_dir.exists() {
        fs::remove_dir_all(dist_dir).ok();
    }
    let binaries_dir = dist_dir.join("binaries");
    let packages_dir = dist_dir.join("packages");
    fs::create_dir_all(&binaries_dir).expect("failed to create dist/binaries directory");
    fs::create_dir_all(&packages_dir).expect("failed to create dist/packages directory");

    // 4. Build Windows native binary
    println!("\n[1/4] Building Windows native release binary...");
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--release");
    let status = cmd.status().expect("failed to execute cargo build");
    if !status.success() {
        eprintln!("Error: Windows build failed.");
        std::process::exit(1);
    }

    // 5. Build Linux musl binary using rust-lld
    println!("\n[2/4] Building Linux target (x86_64-unknown-linux-musl) release binary...");
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--release")
        .arg("--target")
        .arg("x86_64-unknown-linux-musl");
    cmd.env(
        "RUSTFLAGS",
        "-C linker-flavor=ld.lld -C linker=rust-lld",
    );
    let status = cmd.status().expect("failed to execute cargo build for Linux");
    if !status.success() {
        eprintln!("Error: Linux build failed.");
        std::process::exit(1);
    }

    // 6. Generate Debian package (.deb)
    println!("\n[3/4] Generating Debian package (.deb)...");
    let mut cmd = Command::new("cargo");
    cmd.arg("deb")
        .arg("--target")
        .arg("x86_64-unknown-linux-musl")
        .arg("--no-strip");
    cmd.env(
        "RUSTFLAGS",
        "-C linker-flavor=ld.lld -C linker=rust-lld",
    );
    let status = cmd.status().expect("failed to execute cargo deb");
    if !status.success() {
        eprintln!("Error: Debian packaging failed.");
        std::process::exit(1);
    }

    // 7. Generate RPM package (.rpm)
    println!("\n[4/4] Generating RPM package (.rpm)...");
    let mut cmd = Command::new("cargo");
    cmd.arg("generate-rpm")
        .arg("--target")
        .arg("x86_64-unknown-linux-musl");
    cmd.env(
        "RUSTFLAGS",
        "-C linker-flavor=ld.lld -C linker=rust-lld",
    );
    let status = cmd.status().expect("failed to execute cargo generate-rpm");
    if !status.success() {
        eprintln!("Error: RPM packaging failed.");
        std::process::exit(1);
    }

    // 7.5 Generate MSI package (.msi) using cargo wix
    if Path::new("wix").exists() {
        println!("\n[4.5/4.5] Generating MSI package (.msi)...");
        let mut cmd = Command::new("cargo");
        cmd.arg("wix").arg("--no-build");
        let status = cmd.status().expect("failed to execute cargo wix");
        if !status.success() {
            println!("Warning: cargo wix failed. Make sure WiX Toolset is installed.");
        }
    }

    // 8. Copy outputs to dist/
    println!("\n[5/5] Copying packaging results to dist/...");

    // Copy .exe
    let exe_src = Path::new("target")
        .join("release")
        .join(format!("{}.exe", app_name));
    if exe_src.exists() {
        let exe_dest = binaries_dir.join(format!("{}.exe", app_name));
        fs::copy(&exe_src, &exe_dest).expect("failed to copy .exe to dist/binaries/");
        println!("  - Created: {}", exe_dest.display());
    }

    // Copy .deb (cargo-deb outputs under target/<target>/debian/<name>_<version>-1_amd64.deb or similar)
    // Note: Debian revision is usually -1 by default. Let's check both options.
    let deb_filename = format!("{}_{}-1_amd64.deb", app_name, app_version);
    let deb_src = Path::new("target")
        .join("x86_64-unknown-linux-musl")
        .join("debian")
        .join(&deb_filename);
    let deb_filename_alt = format!("{}_{}_amd64.deb", app_name, app_version);
    let deb_src_alt = Path::new("target")
        .join("x86_64-unknown-linux-musl")
        .join("debian")
        .join(&deb_filename_alt);

    if deb_src.exists() {
        let deb_dest = packages_dir.join(&deb_filename);
        fs::copy(&deb_src, &deb_dest).expect("failed to copy .deb to dist/packages/");
        println!("  - Created: {}", deb_dest.display());
    } else if deb_src_alt.exists() {
        let deb_dest = packages_dir.join(&deb_filename_alt);
        fs::copy(&deb_src_alt, &deb_dest).expect("failed to copy .deb to dist/packages/");
        println!("  - Created: {}", deb_dest.display());
    } else {
        // Try searching for any .deb in the debian directory
        let search_dir = Path::new("target")
            .join("x86_64-unknown-linux-musl")
            .join("debian");
        if let Ok(entries) = fs::read_dir(search_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "deb") {
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    let deb_dest = packages_dir.join(filename);
                    fs::copy(&path, &deb_dest).expect("failed to copy .deb to dist/packages/");
                    println!("  - Created: {}", deb_dest.display());
                    break;
                }
            }
        }
    }

    // Copy .rpm (cargo-generate-rpm outputs under target/<target>/generate-rpm/<name>-<version>-1.x86_64.rpm)
    let rpm_filename = format!("{}-{}-1.x86_64.rpm", app_name, app_version);
    let rpm_src = Path::new("target")
        .join("x86_64-unknown-linux-musl")
        .join("generate-rpm")
        .join(&rpm_filename);

    if rpm_src.exists() {
        let rpm_dest = packages_dir.join(&rpm_filename);
        fs::copy(&rpm_src, &rpm_dest).expect("failed to copy .rpm to dist/packages/");
        println!("  - Created: {}", rpm_dest.display());
    } else {
        // Try searching for any .rpm in the generate-rpm directory
        let search_dir = Path::new("target")
            .join("x86_64-unknown-linux-musl")
            .join("generate-rpm");
        if let Ok(entries) = fs::read_dir(search_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "rpm") {
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    let rpm_dest = packages_dir.join(filename);
                    fs::copy(&path, &rpm_dest).expect("failed to copy .rpm to dist/packages/");
                    println!("  - Created: {}", rpm_dest.display());
                    break;
                }
            }
        }
    }

    // Copy .msi if created
    let wix_dir = Path::new("target").join("wix");
    if wix_dir.exists() {
        if let Ok(entries) = fs::read_dir(wix_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "msi") {
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    let msi_dest = packages_dir.join(filename);
                    fs::copy(&path, &msi_dest).expect("failed to copy .msi to dist/packages/");
                    println!("  - Created: {}", msi_dest.display());
                    break;
                }
            }
        }
    }

    // Process packaging templates from packaging/ to dist/packages/
    let packaging_dir = Path::new("packaging");
    if packaging_dir.exists() {
        println!("\n[5/5] Processing packaging templates from packaging/...");
        
        // Process PKGBUILD
        let pkgbuild_src = packaging_dir.join("PKGBUILD");
        if pkgbuild_src.exists() {
            let content = fs::read_to_string(&pkgbuild_src).expect("failed to read PKGBUILD template");
            let new_content = content.replace("TEMPLATE_VERSION", &app_version);
            let pkgbuild_dest = packages_dir.join("PKGBUILD");
            fs::write(&pkgbuild_dest, new_content).expect("failed to write PKGBUILD to dist/packages/");
            println!("  - Created: {}", pkgbuild_dest.display());
        }

        // Process winget.yaml
        let winget_src = packaging_dir.join("winget.yaml");
        if winget_src.exists() {
            let content = fs::read_to_string(&winget_src).expect("failed to read winget.yaml template");
            let new_content = content.replace("TEMPLATE_VERSION", &app_version);
            let winget_dest = packages_dir.join("winget.yaml");
            fs::write(&winget_dest, new_content).expect("failed to write winget.yaml to dist/packages/");
            println!("  - Created: {}", winget_dest.display());
        }
    }

    println!("\nPackaging completed successfully!");
}
