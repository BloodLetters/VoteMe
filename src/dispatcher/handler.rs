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
        let trimmed = username.trim();
        if trimmed.is_empty() || trimmed.len() > 36 {
            return false;
        }

        trimmed.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || character == '_'
                || character == '-'
                || character == '.'
                || character == '+'
                || character == '*'
        })
    }

    pub fn process_incoming_vote(&self, vote: &Vote) -> VoteDispatchResult {
        if !Self::is_valid_username(&vote.username) {
            self.stats.record_failed_vote();
            return VoteDispatchResult {
                broadcast_message: None,
                commands_to_execute: Vec::new(),
                accepted: false,
            };
        }

        self.stats.record_successful_vote(vote.is_v2);

        let clean_player = vote.username.trim();
        let clean_service = if vote.service_name.trim().is_empty() {
            "Votifier"
        } else {
            vote.service_name.trim()
        };

        let broadcast_text = self
            .config
            .broadcast_message
            .replace("{player}", clean_player)
            .replace("{service}", clean_service)
            .replace("{address}", &vote.address)
            .replace("{timestamp}", &vote.timestamp);

        let mut commands = Vec::new();
        for command_template in &self.config.reward_commands {
            let formatted_command = command_template
                .replace("{player}", clean_player)
                .replace("{service}", clean_service)
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
