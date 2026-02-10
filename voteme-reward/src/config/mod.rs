use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub const DEFAULT_YAML_PATH: &str = "plugins/voteme-reward/config.yaml";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub database: DatabaseConfig,

    #[serde(default)]
    pub service: ServiceConfig,

    /// `%player%` placeholder to insert the voting player's username.
    #[serde(default)]
    pub rewards: Vec<String>,

    #[serde(default = "default_log_votes")]
    pub log_votes: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_path")]
    pub path: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    /// Service key registered by the VoteMe plugin.
    #[serde(default = "default_service_key")]
    pub key: String,

    /// Retry delay while waiting for the service to appear.
    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            key: default_service_key(),
            retry_delay_ms: default_retry_delay_ms(),
        }
    }
}

fn default_db_path() -> String {
    "plugins/voteme-reward/voteme.sqlite".to_string()
}

fn default_service_key() -> String {
    "voteme_service".to_string()
}

fn default_retry_delay_ms() -> u64 {
    500
}

fn default_log_votes() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                path: default_db_path(),
            },
            service: ServiceConfig {
                key: default_service_key(),
                retry_delay_ms: default_retry_delay_ms(),
            },
            rewards: Vec::new(),
            log_votes: default_log_votes(),
        }
    }
}

pub struct ConfigManager {
    yaml_path: String,
}

impl ConfigManager {
    pub fn new_default() -> Self {
        Self::new(DEFAULT_YAML_PATH)
    }

    pub fn new(yaml_path: &str) -> Self {
        Self {
            yaml_path: yaml_path.to_string(),
        }
    }

    /// Loads configuration.
    ///
    /// Priority:
    /// 1) If YAML exists: load YAML.
    /// 2) Else: write default YAML, then return defaults.
    pub fn init_config(&self) -> Result<Config, String> {
        let yaml_path = Path::new(&self.yaml_path);

        if let Some(parent) = yaml_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create config dir {parent:?}: {e}"))?;
            }
        }

        if yaml_path.exists() {
            self.load_yaml()
        } else {
            let cfg = Config::default();
            self.save_yaml(&cfg)?;
            Ok(cfg)
        }
    }

    fn load_yaml(&self) -> Result<Config, String> {
        let mut file = fs::File::open(&self.yaml_path)
            .map_err(|e| format!("Failed to open YAML config {}: {e}", self.yaml_path))?;
        let mut s = String::new();
        file.read_to_string(&mut s)
            .map_err(|e| format!("Failed to read YAML config {}: {e}", self.yaml_path))?;

        serde_yaml::from_str(&s)
            .map_err(|e| format!("Invalid YAML config {}: {e}", self.yaml_path))
    }

    fn save_yaml(&self, cfg: &Config) -> Result<(), String> {
        let yaml = serde_yaml::to_string(cfg)
            .map_err(|e| format!("Failed to serialize YAML config: {e}"))?;
        let mut file = fs::File::create(&self.yaml_path)
            .map_err(|e| format!("Failed to create YAML config {}: {e}", self.yaml_path))?;
        file.write_all(yaml.as_bytes())
            .map_err(|e| format!("Failed to write YAML config {}: {e}", self.yaml_path))?;
        Ok(())
    }
}
