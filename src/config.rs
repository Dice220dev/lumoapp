use anyhow::Result;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const APP_NAME: &str = "Lumo";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_PORT: u16 = 47821;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub port: u16,
    pub autostart: bool,
    pub start_minimized: bool,
    pub clipboard_sync: bool,
    pub theme: String,
    pub device_name: String,
    pub inbox_dir: String,
    pub token: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            autostart: false,
            start_minimized: false,
            clipboard_sync: true,
            theme: "dark".into(),
            device_name: computer_name(),
            inbox_dir: default_inbox().to_string_lossy().to_string(),
            token: new_token(),
        }
    }
}

impl Config {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("config.json");
        let loaded = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Config>(&raw).ok());
        let first_run = loaded.is_none();
        let mut cfg = loaded.unwrap_or_default();
        if cfg.token.len() < 16 {
            cfg.token = new_token();
        }
        if cfg.device_name.trim().is_empty() {
            cfg.device_name = computer_name();
        }
        if cfg.inbox_dir.trim().is_empty() {
            cfg.inbox_dir = default_inbox().to_string_lossy().to_string();
        }
        if !matches!(cfg.theme.as_str(), "system" | "light" | "dark") {
            cfg.theme = "dark".into();
        }
        if first_run {
            if let Err(e) = cfg.save(data_dir) {
                eprintln!("could not persist first-run config: {e}");
            }
        }
        cfg
    }

    pub fn save(&self, data_dir: &Path) -> Result<()> {
        fs::create_dir_all(data_dir)?;
        let path = data_dir.join("config.json");
        let tmp = data_dir.join("config.json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
        fs::rename(tmp, path)?;
        Ok(())
    }
}

pub fn new_token() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 32)
}

pub fn computer_name() -> String {
    std::env::var("COMPUTERNAME")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "My PC".into())
}

pub fn default_inbox() -> PathBuf {
    dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Lumo Inbox")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_long_and_unique() {
        let a = new_token();
        let b = new_token();
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
    }
}

pub fn data_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("LUMO_DATA_DIR") {
        if !custom.trim().is_empty() {
            return PathBuf::from(custom);
        }
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_NAME)
}
