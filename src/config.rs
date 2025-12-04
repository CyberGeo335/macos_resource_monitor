use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub disk_usage_percent: f32,
    pub net_in_kbps: f32,
    pub net_out_kbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub collection_interval_secs: u64,
    pub log_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub service: ServiceConfig,
    pub thresholds: Thresholds,
}

fn default_base_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        Path::new(&home)
            .join("Library")
            .join("Application Support")
            .join("MacResourceMonitor")
    } else {
        // крайний случай – /tmp
        Path::new("/tmp").join("MacResourceMonitor")
    }
}

pub fn default_config_path() -> PathBuf {
    default_base_dir().join("config.toml")
}

pub fn ensure_base_dir() -> std::io::Result<PathBuf> {
    let dir = default_base_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub fn load_or_create_default() -> Result<Config, ConfigError> {
    let base_dir = ensure_base_dir()?;
    let config_path = base_dir.join("config.toml");

    if config_path.exists() {
        let data = fs::read_to_string(&config_path)?;
        let cfg: Config = toml::from_str(&data)?;
        Ok(cfg)
    } else {
        let log_path = base_dir
            .join("logs")
            .join(format!("metrics-{}.log", Utc::now().format("%Y%m%d")));
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let cfg = Config {
            service: ServiceConfig {
                collection_interval_secs: 5,
                log_file_path: log_path.to_string_lossy().into_owned(),
            },
            thresholds: Thresholds {
                cpu_usage_percent: 80.0,
                memory_usage_percent: 80.0,
                disk_usage_percent: 90.0,
                net_in_kbps: 1_000_000.0, // 1 Гбит/с
                net_out_kbps: 1_000_000.0,
            },
        };

        let toml_str = toml::to_string_pretty(&cfg)?;
        let mut file = fs::File::create(&config_path)?;
        file.write_all(toml_str.as_bytes())?;

        Ok(cfg)
    }
}
