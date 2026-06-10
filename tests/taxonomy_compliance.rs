use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn test_taxonomy_compliance() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files_checked = 0;
    let mut violations = Vec::new();

    fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, files)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    let mut rs_files = Vec::new();
    visit_dirs(&src_dir, &mut rs_files).expect("failed to walk src directory");

    for file_path in rs_files {
        let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        
        // Skip entry points, binaries, shims, and system-generated files
        if file_name == "lib.rs" 
            || file_name == "main.rs" 
            || file_name == "interface.rs"
            || file_name == "lifecycle.rs"
            || file_name == "platform.rs"
            || file_name == "role.rs"
            || file_name == "screensaver_runtime.rs"
            || file_path.to_string_lossy().contains("src/bin/")
            || file_path.to_string_lossy().contains(".system_generated")
        {
            continue;
        }

        let relative_path = file_path.strip_prefix(&src_dir).unwrap();
        let path_str = relative_path.to_string_lossy().replace('\\', "/");
        
        let layer = if path_str.starts_with("core/") || path_str == "core.rs" {
            "core"
        } else if path_str.starts_with("toolkit/") || path_str == "platform.rs" {
            "toolkit"
        } else if path_str.starts_with("ui/") || path_str == "interface.rs" {
            "ui"
        } else if path_str.starts_with("apps/") || path_str == "lifecycle.rs" {
            "apps"
        } else if path_str.starts_with("screensavers/") || path_str == "role.rs" {
            "screensavers"
        } else {
            continue; // Skip top-level files that are not part of taxonomy layers
        };

        // Prohibited dependencies for each layer to maintain clean separation
        let prohibited_layers: &[&str] = match layer {
            "core" => &["toolkit", "ui", "apps", "screensavers"],
            "toolkit" => &["ui", "apps", "screensavers"],
            "ui" => &["apps", "screensavers"],
            "apps" => &["screensavers"],
            "screensavers" => &[],
            _ => &[],
        };

        if prohibited_layers.is_empty() {
            continue;
        }

        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("failed to read file: {:?}", file_path));

        // Strip comments to prevent flagging example code or documentation notes
        let cleaned_content = strip_comments(&content);

        for prohibited in prohibited_layers {
            let crate_pattern = format!("crate::{}", prohibited);
            let super_pattern = format!("super::{}", prohibited);
            
            if cleaned_content.contains(&crate_pattern) || cleaned_content.contains(&super_pattern) {
                violations.push(format!(
                    "File: {} (Layer: {})\n  Violates taxonomy rules by importing/referencing: '{}'",
                    path_str, layer, prohibited
                ));
            }
        }

        files_checked += 1;
    }

    println!("Taxonomy compliance checked {} files.", files_checked);

    if !violations.is_empty() {
        let error_message = format!(
            "Taxonomy dependency violations detected! The 4-layer taxonomy enforces unidirectional, decoupled layering.\n\n{}",
            violations.join("\n\n")
        );
        panic!("{}", error_message);
    }
}

fn strip_comments(content: &str) -> String {
    let mut result = String::new();
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    chars.next();
                    for lc in chars.by_ref() {
                        if lc == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                    continue;
                } else if next == '*' {
                    chars.next();
                    while let Some(bc) = chars.next() {
                        if bc == '*' {
                            if let Some(&next_next) = chars.peek() {
                                if next_next == '/' {
                                    chars.next();
                                    break;
                                }
                            }
                        }
                    }
                    continue;
                }
            }
        }
        result.push(c);
    }
    result
}

#[test]
fn test_taxonomy_features_compile() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    // We test that each key taxonomy feature compiles independently
    let feature_sets = &[
        vec!["--no-default-features"],
        vec!["--no-default-features", "--features", "interface-app"],
        vec!["--no-default-features", "--features", "interface-api"],
        vec!["--no-default-features", "--features", "lifecycle-foreground"],
        vec!["--no-default-features", "--features", "platform-native"],
        vec!["--no-default-features", "--features", "role-system"],
        vec!["--no-default-features", "--features", "role-application"],
        vec!["--all-features"],
    ];

    for args in feature_sets {
        let status = std::process::Command::new(&cargo)
            .arg("check")
            .args(args)
            .status()
            .expect("Failed to run cargo check");
        assert!(status.success(), "Cargo check failed for args: {:?}", args);
    }
}
