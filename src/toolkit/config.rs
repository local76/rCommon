//! Generic configuration parser and writer for yaml-like key-value storage.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment - Native).

use std::io;
use std::path::PathBuf;

/// Trait to be implemented by application-specific configuration field structures.
pub trait ConfigFields: Default {
    /// Update field matching `key` with value `val`.
    fn parse_field(&mut self, key: &str, val: &str);
    /// Serialize fields into a list of key-value string pairs.
    fn serialize_fields(&self) -> Vec<(String, String)>;
}

/// Generic application configuration wrapper.
#[derive(Debug, Clone)]
pub struct AppConfig<T: ConfigFields> {
    pub fields: T,
}

impl<T: ConfigFields> AppConfig<T> {
    /// Resolves path to `%APPDATA%\<app_name>\<filename>`
    pub fn config_path(app_name: &str, filename: &str) -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var("APPDATA").ok().map(|appdata| {
                std::path::PathBuf::from(appdata)
                    .join("local76")
                    .join(app_name)
                    .join(filename)
            })
        }
        #[cfg(not(windows))]
        {
            let base = std::env::var("XDG_CONFIG_HOME")
                .ok()
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var("HOME").ok().map(|home| {
                        PathBuf::from(home).join(".config")
                    })
                });
            base.map(|b| b.join("local76").join(app_name).join(filename))
        }
    }

    /// Load config from file, falling back to defaults on failure or missing file.
    pub fn load(app_name: &str, filename: &str) -> Self {
        let Some(path) = Self::config_path(app_name, filename) else {
            return Self { fields: T::default() };
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self { fields: T::default() };
        };

        let mut fields = T::default();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim();
                let val = line[pos + 1..].trim();
                fields.parse_field(key, val);
            }
        }
        Self { fields }
    }

    /// Save current config properties to file.
    pub fn save(&self, app_name: &str, filename: &str, header: &str) -> io::Result<()> {
        let Some(path) = Self::config_path(app_name, filename) else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        if !header.is_empty() {
            for line in header.lines() {
                let line = line.trim();
                if line.starts_with('#') {
                    content.push_str(line);
                } else {
                    content.push_str(&format!("# {}", line));
                }
                content.push('\n');
            }
            content.push('\n');
        }

        for (k, v) in self.fields.serialize_fields() {
            content.push_str(&format!("{}: {}\n", k, v));
        }

        crate::core::write_file_atomic(path, content)
    }
}
