use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Config {
    pub discord_client_id: Option<String>,
}

impl Config {
    fn config_dir() -> Option<PathBuf> {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg).join("anily"));
        }
        if let Some(home) = std::env::var_os("HOME") {
            return Some(PathBuf::from(home).join(".config").join("anily"));
        }
        None
    }

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
