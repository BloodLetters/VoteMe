use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::crypto::hmac::generate_service_token;
use crate::network::throttle::ThrottleOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteSiteConfig {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotifierConfig {
    pub host: String,
    pub port: u16,
    pub tokens: HashMap<String, String>,
    pub disable_v1: bool,
    pub broadcast_message: String,
    pub reward_commands: Vec<String>,
    pub vote_sites: Vec<VoteSiteConfig>,
    pub throttle_enabled: bool,
}

impl Default for VotifierConfig {
    fn default() -> Self {
        let mut tokens = HashMap::new();
        tokens.insert("default".to_string(), generate_service_token());

        Self {
            host: "0.0.0.0".to_string(),
            port: 8192,
            tokens,
            disable_v1: false,
            broadcast_message: "§a[Vote] §f{player} just voted on §e{service}§f!".to_string(),
            reward_commands: vec![
                "give {player} diamond 1".to_string(),
                "tellraw {player} [{\"text\":\"Thank you for voting! Enjoy your reward.\",\"color\":\"green\"}]".to_string(),
            ],
            vote_sites: vec![
                VoteSiteConfig {
                    name: "PlanetMinecraft".to_string(),
                    url: "https://www.planetminecraft.com/".to_string(),
                },
                VoteSiteConfig {
                    name: "Minecraft-MP".to_string(),
                    url: "https://minecraft-mp.com/".to_string(),
                },
            ],
            throttle_enabled: true,
        }
    }
}

impl VotifierConfig {
    pub fn to_throttle_options(&self) -> ThrottleOptions {
        ThrottleOptions {
            enabled: self.throttle_enabled,
            ..Default::default()
        }
    }
}
