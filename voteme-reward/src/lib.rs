use std::{sync::Arc, time::Duration};

use pumpkin::command::CommandSender;
use pumpkin::plugin::Context;
use pumpkin_api_macros::{plugin_impl, plugin_method};
use voteme_api::VoteService;

mod config;
mod storage;
use storage::database::Database;

#[plugin_method]
fn on_load(&mut self, server: Arc<Context>) -> Result<(), String> {
    server.init_log();
    log::info!("VoteReward plugin loading...");

    // PluginManager::wait_for_plugin(&server.plugin_manager, "voteme");

    let cfg = config::ConfigManager::new_default().init_config()?;
    log::info!("VoteReward config loaded (yaml: {})", config::DEFAULT_YAML_PATH);

    let db = Arc::new(Database::open(&cfg.database.path)?);
    log::info!("VoteReward sqlite ready: {:?}", db.path());

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create Tokio runtime: {e}"))?;

    rt.block_on(async move {

        let retry_delay = Duration::from_millis(cfg.service.retry_delay_ms);
        let service_key = cfg.service.key;
        let log_votes = cfg.log_votes;
        let rewards = Arc::new(cfg.rewards);
        let pumpkin_server = Arc::clone(&server.server);

        loop {
            if let Some(service) = server.get_service::<VoteService>(&service_key).await {
                log::info!("✅ VoteService found! Registering reward listener.");

                let db = Arc::clone(&db);
                let rewards = Arc::clone(&rewards);
                let pumpkin_server = Arc::clone(&pumpkin_server);

                service.on_vote(move |vote| {
                    if log_votes {
                        log::info!("Rewarding player: {}", vote.username);
                    }

                    if let Err(e) = db.insert_vote(&vote) {
                        log::error!("Failed to persist vote to sqlite: {e}");
                    }

                    if rewards.is_empty() {
                        return;
                    }

                    let rewards = Arc::clone(&rewards);
                    let pumpkin_server = Arc::clone(&pumpkin_server);
                    let player_name = vote.username.clone();

                    tokio::spawn(async move {
                        let sender = CommandSender::Console;
                        let dispatcher = pumpkin_server.command_dispatcher.read().await;

                        for raw in rewards.iter() {
                            let cmd = raw.replace("%player%", &player_name);
                            let cmd = cmd.trim();
                            if cmd.is_empty() {
                                continue;
                            }

                            let cmd = cmd.strip_prefix('/').unwrap_or(cmd);
                            dispatcher
                                .handle_command(&sender, pumpkin_server.as_ref(), cmd)
                                .await;
                        }
                    });
                });

                log::info!("VoteReward listener registered.");
                break;
            }

            log::warn!("VoteService not found yet, retrying...");
            tokio::time::sleep(retry_delay).await;
        }

        Ok(())
    })
}


#[plugin_impl]
pub struct VoteReward;

impl VoteReward {
    pub fn new() -> Self {
        VoteReward
    }
}

impl Default for VoteReward {
    fn default() -> Self {
        Self::new()
    }
}