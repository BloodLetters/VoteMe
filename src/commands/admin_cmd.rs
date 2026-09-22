use std::sync::Arc;

use pumpkin_plugin_api::command::{Command, CommandError, CommandNode, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::Server;

use crate::config::settings::VotifierConfig;
use crate::model::stats::VoteStatistics;

pub struct VotifierAdminCommandHandler {
    config: Arc<VotifierConfig>,
    stats: Arc<VoteStatistics>,
}

impl VotifierAdminCommandHandler {
    pub fn new(config: Arc<VotifierConfig>, stats: Arc<VoteStatistics>) -> Self {
        Self { config, stats }
    }

    pub fn build_command(config: Arc<VotifierConfig>, stats: Arc<VoteStatistics>) -> Command {
        let status_node = CommandNode::literal("status")
            .execute(Self::new(Arc::clone(&config), Arc::clone(&stats)));

        let help_node = CommandNode::literal("help")
            .execute(VotifierAdminHelpHandler);

        Command::new(
            &["voteme".to_string(), "votifier".to_string()],
            "Votifier administrator commands and statistics",
        )
        .then(status_node)
        .then(help_node)
        .execute(Self::new(config, stats))
    }
}

impl CommandHandler for VotifierAdminCommandHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let mut message = String::from("§6=== §eVotifier Plugin Status §6===\n");
        message.push_str(&format!("§7Listening On: §a{}:{}\n", self.config.host, self.config.port));
        message.push_str(&format!("§7Total Inbound: §e{}\n", self.stats.total_received()));
        message.push_str(&format!("§7Valid Votes: §a{}\n", self.stats.valid_votes()));
        message.push_str(&format!("  §7- V1 (RSA): §b{}\n", self.stats.v1_votes()));
        message.push_str(&format!("  §7- V2 (Token): §b{}\n", self.stats.v2_votes()));
        message.push_str(&format!("§7Failed/Rejected: §c{}\n", self.stats.failed_votes()));
        message.push_str("§6=================================");

        sender.send_message(TextComponent::from_legacy_string(&message));
        Ok(0)
    }
}

pub struct VotifierAdminHelpHandler;

impl CommandHandler for VotifierAdminHelpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let mut message = String::from("§6=== §eVotifier Admin Commands §6===\n");
        message.push_str("§e/voteme §7- Show plugin status and statistics\n");
        message.push_str("§e/voteme status §7- Show runtime listener statistics\n");
        message.push_str("§e/voteme help §7- Show administrator help message\n");
        message.push_str("§6==================================");

        sender.send_message(TextComponent::from_legacy_string(&message));
        Ok(0)
    }
}
