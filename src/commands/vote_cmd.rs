use std::sync::Arc;

use pumpkin_plugin_api::command::{Command, CommandError, CommandNode, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::Server;

use crate::config::settings::VotifierConfig;

pub struct VotePlayerCommandHandler {
    config: Arc<VotifierConfig>,
}

impl VotePlayerCommandHandler {
    pub fn new(config: Arc<VotifierConfig>) -> Self {
        Self { config }
    }

    pub fn build_command(config: Arc<VotifierConfig>) -> Command {
        let list_node = CommandNode::literal("list")
            .execute(Self::new(Arc::clone(&config)));

        let sites_node = CommandNode::literal("sites")
            .execute(Self::new(Arc::clone(&config)));

        Command::new(
            &["vote".to_string(), "votes".to_string()],
            "Displays server voting links and rewards info",
        )
        .then(list_node)
        .then(sites_node)
        .execute(Self::new(config))
    }
}

impl CommandHandler for VotePlayerCommandHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let mut response = String::from("§6=== §eServer Voting Sites §6===\n");
        response.push_str("§7Vote daily to support the server and earn rewards!\n");

        if self.config.vote_sites.is_empty() {
            response.push_str("§7No voting sites configured currently.\n");
        } else {
            for (index, site) in self.config.vote_sites.iter().enumerate() {
                response.push_str(&format!("§a{}. §e{} §7- §b{}\n", index + 1, site.name, site.url));
            }
        }
        response.push_str("§6==============================");

        sender.send_message(TextComponent::from_legacy_string(&response));
        Ok(0)
    }
}
