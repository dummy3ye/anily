// src/config.rs
use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Config {
    /// Discord application client ID for Rich Presence
    pub discord_client_id: Option<String>,
}

impl Config {
    /// Resolve the config directory: `$XDG_CONFIG_HOME/anily/` or `~/.config/anily/`.
    fn config_dir() -> Option<PathBuf> {
        // Prefer XDG_CONFIG_HOME directly so the path is always ~/.config/anily
        // regardless of what the `directories` crate infers for the qualifier.
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg).join("anily"));
        }
        if let Some(home) = std::env::var_os("HOME") {
            return Some(PathBuf::from(home).join(".config").join("anily"));
        }
        None
    }

    /// Load configuration from `~/.config/anily/config.toml`.
    /// Returns a default (empty) config if the file does not exist.
    pub fn load() -> Result<Self> {
        if let Some(dir) = Self::config_dir() {
            let path = dir.join("config.toml");
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let cfg: Config = toml::from_str(&content)?;
                return Ok(cfg);
            }
        }
        Ok(Config::default())
    }
}
