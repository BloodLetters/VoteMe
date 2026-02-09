use std::{sync::Arc, time::Duration};

use pumpkin::plugin::Context;
use pumpkin_api_macros::{plugin_impl, plugin_method};
use voteme_api::VoteService;

#[plugin_method]
fn on_load(&mut self, server: Arc<Context>) -> Result<(), String> {
    server.init_log();
    log::info!("VoteReward plugin loading...");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create Tokio runtime: {e}"))?;

    rt.block_on(async move {

        loop {
            if let Some(service) = server.get_service::<VoteService>("voteme_service").await {
                log::info!("✅ VoteService found! Registering reward listener.");

                service.on_vote(|vote| {
                    log::info!("Rewarding player: {}", vote.username);
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