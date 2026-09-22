use std::fs;
use std::path::Path;

use crate::config::settings::VotifierConfig;
use crate::error::{VotifierError, VotifierResult};

pub const CONFIG_FILE_NAME: &str = "config.json";

pub fn load_or_create_config(data_folder: &Path) -> VotifierResult<VotifierConfig> {
    if !data_folder.exists() {
        fs::create_dir_all(data_folder).map_err(|e| {
            VotifierError::ConfigurationError(format!(
                "Failed to create data folder {}: {e}",
                data_folder.display()
            ))
        })?;
    }

    let config_path = data_folder.join(CONFIG_FILE_NAME);

    if config_path.exists() {
        let content = fs::read_to_string(&config_path).map_err(|e| {
            VotifierError::ConfigurationError(format!(
                "Failed to read config file {}: {e}",
                config_path.display()
            ))
        })?;

        let config: VotifierConfig = serde_json::from_str(&content).map_err(|e| {
            VotifierError::ConfigurationError(format!(
                "Invalid JSON syntax in {}: {e}",
                config_path.display()
            ))
        })?;

        Ok(config)
    } else {
        let default_config = VotifierConfig::default();
        save_config(data_folder, &default_config)?;
        Ok(default_config)
    }
}

pub fn save_config(data_folder: &Path, config: &VotifierConfig) -> VotifierResult<()> {
    let config_path = data_folder.join(CONFIG_FILE_NAME);
    let serialized = serde_json::to_string_pretty(config)?;

    fs::write(&config_path, serialized.as_bytes()).map_err(|e| {
        VotifierError::ConfigurationError(format!(
            "Failed to save config to {}: {e}",
            config_path.display()
        ))
    })?;

    Ok(())
}
