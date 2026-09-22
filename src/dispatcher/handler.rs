use std::sync::Arc;

use crate::config::settings::VotifierConfig;
use crate::model::stats::VoteStatistics;
use crate::model::Vote;

pub struct VoteDispatcher {
    config: Arc<VotifierConfig>,
    stats: Arc<VoteStatistics>,
}

impl VoteDispatcher {
    pub fn new(config: Arc<VotifierConfig>, stats: Arc<VoteStatistics>) -> Self {
        Self { config, stats }
    }

    pub fn is_valid_username(username: &str) -> bool {
        if username.is_empty() || username.len() > 16 {
            return false;
        }

        username
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    }

    pub fn process_incoming_vote(&self, vote: &Vote) -> VoteDispatchResult {
        if !Self::is_valid_username(&vote.username) || vote.service_name.trim().is_empty() {
            self.stats.record_failed_vote();
            return VoteDispatchResult {
                broadcast_message: None,
                commands_to_execute: Vec::new(),
                accepted: false,
            };
        }

        self.stats.record_successful_vote(vote.is_v2);

        let broadcast_text = self
            .config
            .broadcast_message
            .replace("{player}", &vote.username)
            .replace("{service}", &vote.service_name)
            .replace("{address}", &vote.address)
            .replace("{timestamp}", &vote.timestamp);

        let mut commands = Vec::new();
        for command_template in &self.config.reward_commands {
            let formatted_command = command_template
                .replace("{player}", &vote.username)
                .replace("{service}", &vote.service_name)
                .replace("{address}", &vote.address)
                .replace("{timestamp}", &vote.timestamp);
            commands.push(formatted_command);
        }

        VoteDispatchResult {
            broadcast_message: Some(broadcast_text),
            commands_to_execute: commands,
            accepted: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoteDispatchResult {
    pub broadcast_message: Option<String>,
    pub commands_to_execute: Vec<String>,
    pub accepted: bool,
}
