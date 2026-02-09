use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub const DEFAULT_CONFIG_PATH: &str = "plugins/VoteMe/Config.toml";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_rsa_bits")]
    pub rsa_bits: u32,

    #[serde(default = "default_debug")]
    pub debug: bool,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8192
}

fn default_rsa_bits() -> u32 {
    2048
}

fn default_debug() -> bool {
    false
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: default_host(),
            port: default_port(),
            rsa_bits: default_rsa_bits(),
            debug: default_debug(),
        }
    }
}

pub struct ConfigManager {
    config_file: String,
}

impl ConfigManager {
    pub fn new_default() -> Self {
        Self::new(DEFAULT_CONFIG_PATH)
    }

    pub fn new(config_file: &str) -> Self {
        ConfigManager {
            config_file: config_file.to_string(),
        }
    }

    pub async fn init_config(&mut self, config: &mut Config) -> Result<(), String> {
        let config_path = Path::new(&self.config_file);

        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        if !config_path.exists() {
            self.save_config(config).await?;
            log::info!("Created new config file at: {}", self.config_file);
        } else {
            self.load_config(config).await?;
            log::info!("Loaded config file from: {}", self.config_file);
        }
        Ok(())
    }

    pub async fn load_config(&self, config: &mut Config) -> Result<(), String> {
        let mut file = std::fs::File::open(&self.config_file).map_err(|e| e.to_string())?;
        let mut config_str = String::new();
        file.read_to_string(&mut config_str)
            .map_err(|e| e.to_string())?;
        *config = toml::from_str(&config_str).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn save_config(&self, config: &Config) -> Result<(), String> {
        let config_path = Path::new(&self.config_file);

        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut file = std::fs::File::create(&self.config_file).map_err(|e| e.to_string())?;
        let config_str = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
        file.write_all(config_str.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }
}
