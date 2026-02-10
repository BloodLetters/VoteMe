use std::{sync::Arc, time::Duration};

use pumpkin::plugin::{Context, PluginManager};
use pumpkin_api_macros::{plugin_impl, plugin_method};
use voteme_api::VoteService;

mod storage;
use storage::database::Database;

#[plugin_method]
fn on_load(&mut self, server: Arc<Context>) -> Result<(), String> {
    server.init_log();
    log::info!("VoteReward plugin loading...");

    // PluginManager::wait_for_plugin(&server.plugin_manager, "voteme");

    let db = Arc::new(Database::open("plugins/voteme-reward/voteme.sqlite")?);
    log::info!("VoteReward sqlite ready: {:?}", db.path());

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create Tokio runtime: {e}"))?;

    rt.block_on(async move {

        loop {
            if let Some(service) = server.get_service::<VoteService>("voteme_service").await {
                log::info!("✅ VoteService found! Registering reward listener.");

                let db = Arc::clone(&db);

                service.on_vote(move |vote| {
                    log::info!("Rewarding player: {}", vote.username);

                    if let Err(e) = db.insert_vote(&vote) {
                        log::error!("Failed to persist vote to sqlite: {e}");
                    }
                });

                log::info!("VoteReward listener registered.");
                break;
            }

            log::warn!("VoteService not found yet, retrying in 500ms...");
            tokio::time::sleep(Duration::from_millis(500)).await;
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