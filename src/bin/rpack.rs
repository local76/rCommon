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

    // 3. Create dist directory
    let dist_dir = Path::new("dist");
    if dist_dir.exists() {
        fs::remove_dir_all(dist_dir).ok();
    }
    fs::create_dir_all(dist_dir).expect("failed to create dist directory");

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

    // 8. Copy outputs to dist/
    println!("\n[5/5] Copying packaging results to dist/...");

    // Copy .exe
    let exe_src = Path::new("target")
        .join("release")
        .join(format!("{}.exe", app_name));
    if exe_src.exists() {
        let exe_dest = dist_dir.join(format!("{}.exe", app_name));
        fs::copy(&exe_src, &exe_dest).expect("failed to copy .exe to dist/");
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
        let deb_dest = dist_dir.join(&deb_filename);
        fs::copy(&deb_src, &deb_dest).expect("failed to copy .deb to dist/");
        println!("  - Created: {}", deb_dest.display());
    } else if deb_src_alt.exists() {
        let deb_dest = dist_dir.join(&deb_filename_alt);
        fs::copy(&deb_src_alt, &deb_dest).expect("failed to copy .deb to dist/");
        println!("  - Created: {}", deb_dest.display());
    } else {
        // Try searching for any .deb in the debian directory
        let search_dir = Path::new("target")
            .join("x86_64-unknown-linux-musl")
            .join("debian");
        if let Ok(entries) = fs::read_dir(search_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "deb") {
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    let deb_dest = dist_dir.join(filename);
                    fs::copy(&path, &deb_dest).expect("failed to copy .deb to dist/");
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
        let rpm_dest = dist_dir.join(&rpm_filename);
        fs::copy(&rpm_src, &rpm_dest).expect("failed to copy .rpm to dist/");
        println!("  - Created: {}", rpm_dest.display());
    } else {
        // Try searching for any .rpm in the generate-rpm directory
        let search_dir = Path::new("target")
            .join("x86_64-unknown-linux-musl")
            .join("generate-rpm");
        if let Ok(entries) = fs::read_dir(search_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "rpm") {
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    let rpm_dest = dist_dir.join(filename);
                    fs::copy(&path, &rpm_dest).expect("failed to copy .rpm to dist/");
                    println!("  - Created: {}", rpm_dest.display());
                    break;
                }
            }
        }
    }

    println!("\nPackaging completed successfully!");
}
