use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{Error, Result};

/// Configuration for better-cp
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub defaults: Defaults,
    pub behavior: Behavior,
    pub performance: Performance,
    pub ui: UiConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Defaults {
    pub overwrite: String,  // "prompt", "never", "always", "smart"
    pub resume: bool,
    pub verify: String,    // "none", "fast", "full"
    pub parallel: usize,
    pub sparse: bool,
    pub reflink: String,   // "auto", "always", "never"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Behavior {
    pub follow_symlinks: bool,
    pub preserve_times: bool,
    pub preserve_permissions: bool,
    pub atomic: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Performance {
    pub buffer_size: String,
    pub chunk_size: String,
    pub resume_threshold: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiConfig {
    pub color: bool,
    pub progress_style: String,  // "bars", "minimal", "json"
    pub show_per_file: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            defaults: Defaults {
                overwrite: "prompt".to_string(),
                resume: true,
                verify: "fast".to_string(),
                parallel: 4,
                sparse: true,
                reflink: "auto".to_string(),
            },
            behavior: Behavior {
                follow_symlinks: false,
                preserve_times: true,
                preserve_permissions: true,
                atomic: true,
            },
            performance: Performance {
                buffer_size: "64M".to_string(),
                chunk_size: "100M".to_string(),
                resume_threshold: "100M".to_string(),
            },
            ui: UiConfig {
                color: true,
                progress_style: "bars".to_string(),
                show_per_file: false,
            },
        }
    }
}

impl Config {
    /// Load config from standard locations
    pub fn load() -> Result<Config> {
        // Try ~/.config/better-cp/config.toml (Linux/macOS)
        // Try %APPDATA%\better-cp\config.toml (Windows)

        if let Ok(config_dir) = std::env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(config_dir).join("better-cp/config.toml");
            if path.exists() {
                return Self::from_file(&path);
            }
        }

        if let Some(home_dir) = dirs::home_dir() {
            let path = home_dir.join(".config/better-cp/config.toml");
            if path.exists() {
                return Self::from_file(&path);
            }
        }

        // Fall back to defaults if no config found
        Ok(Self::default())
    }

    /// Load config from specific file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::ConfigError(format!("Failed to read config: {}", e)))?;
        toml::from_str(&content)
            .map_err(|e| Error::ConfigError(format!("Failed to parse TOML: {}", e)))
    }

    /// Save config to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::ConfigError(format!("Failed to serialize config: {}", e)))?;
        fs::write(path, content)
            .map_err(Error::Io)
    }

    /// Parse buffer size string (e.g., "64M", "1G") to bytes
    pub fn parse_size(size_str: &str) -> Result<u64> {
        let size_str = size_str.trim().to_uppercase();
        let (num_part, unit_part) = size_str.split_at(
            size_str.find(|c: char| c.is_alphabetic())
                .unwrap_or(size_str.len())
        );

        let num: u64 = num_part.trim().parse()
            .map_err(|_| Error::ConfigError(format!("Invalid size: {}", size_str)))?;

        let multiplier = match unit_part {
            "B" => 1,
            "K" | "KB" => 1024,
            "M" | "MB" => 1024 * 1024,
            "G" | "GB" => 1024 * 1024 * 1024,
            "T" | "TB" => 1024 * 1024 * 1024 * 1024,
            "" => 1,
            _ => return Err(Error::ConfigError(format!("Unknown size unit: {}", unit_part))),
        };

        Ok(num * multiplier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.defaults.overwrite, "prompt");
        assert!(config.behavior.atomic);
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(Config::parse_size("64M").unwrap(), 64 * 1024 * 1024);
        assert_eq!(Config::parse_size("1G").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(Config::parse_size("512").unwrap(), 512);
    }
}
