pub mod commands;
pub mod config;
pub mod crypto;
pub mod dispatcher;
pub mod error;
pub mod model;
pub mod network;
pub mod protocol;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use pumpkin_plugin_api::permission::{Permission, PermissionDefault, PermissionLevel};
use pumpkin_plugin_api::permissions;
use pumpkin_plugin_api::{Context, Plugin, PluginMetadata, Result, register_plugin};

use crate::commands::{VotePlayerCommandHandler, VotifierAdminCommandHandler};
use crate::config::load_or_create_config;
use crate::crypto::RsaKeyManager;
use crate::dispatcher::VoteDispatcher;
use crate::model::VoteStatistics;
use crate::network::{VoteReceiver, VoteThrottleService};

pub struct VotifierPlugin {
    receiver: Mutex<Option<VoteReceiver>>,
    stats: Arc<VoteStatistics>,
}

impl Plugin for VotifierPlugin {
    fn new() -> Self {
        Self {
            receiver: Mutex::new(None),
            stats: Arc::new(VoteStatistics::new()),
        }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "voteme".into(),
            version: "0.1.0".into(),
            authors: vec!["SirAshesh".into()],
            description: "Votifier vote listener for Pumpkin server.".into(),
            dependencies: vec![],
            permissions: vec![
                permissions::FS_READ_DATA.into(),
                permissions::FS_WRITE_DATA.into(),
                permissions::NETWORK_TCP_BIND.into(),
                permissions::NETWORK_TCP.into(),
            ],
        }
    }

    fn on_load(&self, context: Context) -> Result<()> {
        let data_folder = PathBuf::from(context.get_data_folder());

        let config = Arc::new(
            load_or_create_config(&data_folder).map_err(|e| format!("Config load error: {e}"))?,
        );

        let rsa_directory = data_folder.join("rsa");
        let key_manager = Arc::new(
            RsaKeyManager::load_or_generate_in_directory(&rsa_directory)
                .map_err(|e| format!("RSA key initialization error: {e}"))?,
        );

        let throttle_service = Arc::new(VoteThrottleService::new(config.to_throttle_options()));

        let dispatcher = Arc::new(VoteDispatcher::new(
            Arc::clone(&config),
            Arc::clone(&self.stats),
        ));

        let mut receiver = VoteReceiver::new(&config.host, config.port);
        let tokens_map = Arc::new(config.tokens.clone());

        let server_instance = context.get_server();
        let vote_dispatcher = Arc::clone(&dispatcher);

        let vote_callback = Arc::new(move |vote| {
            let result = vote_dispatcher.process_incoming_vote(&vote);
            if result.accepted {
                if let Some(broadcast_message) = result.broadcast_message {
                    server_instance.broadcast(&broadcast_message);
                }
                for command in result.commands_to_execute {
                    server_instance.execute_command(
                        &command,
                        pumpkin_plugin_api::server::CommandSender::Console,
                    );
                }
            }
        });

        receiver
            .start(
                Arc::clone(&key_manager),
                tokens_map,
                Arc::clone(&throttle_service),
                config.disable_v1,
                vote_callback,
            )
            .map_err(|e| format!("Failed to start Votifier listener: {e}"))?;

        context.register_permission(&Permission {
            node: "voteme:vote".into(),
            description: "Allows players to view voting links with /vote".into(),
            default: PermissionDefault::Allow,
            children: vec![],
        })?;

        context.register_permission(&Permission {
            node: "voteme:admin".into(),
            description: "Allows administrators to manage Votifier with /voteme".into(),
            default: PermissionDefault::Op(PermissionLevel::Three),
            children: vec![],
        })?;

        let vote_command = VotePlayerCommandHandler::build_command(Arc::clone(&config));
        context.register_command(vote_command, "voteme:vote");

        let admin_command = VotifierAdminCommandHandler::build_command(
            Arc::clone(&config),
            Arc::clone(&self.stats),
        );
        context.register_command(admin_command, "voteme:admin");

        *self.receiver.lock().unwrap() = Some(receiver);
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<()> {
        if let Some(mut receiver) = self.receiver.lock().unwrap().take() {
            receiver.shutdown();
        }
        Ok(())
    }
}

register_plugin!(VotifierPlugin);

#[cfg(test)]
mod tests;
