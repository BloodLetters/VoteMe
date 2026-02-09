use std::{path::Path, sync::{Arc, Mutex}};

use pumpkin::plugin::{Context};
use pumpkin_api_macros::{plugin_impl, plugin_method};
use tokio::{io::AsyncWriteExt, net::TcpListener};

use file::config::ConfigManager;
use net::vote_handler::VoteHandler;
use crypto::{RSAIO, RSAKeyGen};
use file::Config;
use voteme_api::{Vote, VoteService};

mod crypto;
mod file;
mod net;
mod parser;
pub mod vote;

#[plugin_method]
async fn on_load(&mut self, server: Arc<Context>) -> Result<(), String> {
    server.init_log();
    log::info!("VoteMe plugin loading...");

    let mut config = Config::default();
    let mut config_manager = ConfigManager::new_default();
    config_manager.init_config(&mut config).await?;

    // Avoid moving `config` into the background thread (it would prevent later use).
    let host = config.host.clone();
    let port = config.port;
    let debug = config.debug;

    let privkey = if Path::new("plugins/VoteMe/rsa/private.key").exists() {
        RSAIO::load_private("private.key")
    } else {
        let (privkey, pubkey) = RSAKeyGen::generate(config.rsa_bits as usize);
        RSAIO::save_private(&privkey, "private.key");
        RSAIO::save_public(&pubkey, "public.key");
        privkey
    };

    let vote_service = Arc::new(VoteService::new());
    server
        .register_service("voteme_service", vote_service.clone())
        .await;

    let server_ctx = server.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            let bind_addr = format!("{}:{}", host, port);

            let listener = match TcpListener::bind(&bind_addr).await {
                Ok(l) => l,
                Err(e) => {
                    log::error!("Bind failed: {}", e);
                    return;
                }
            };

            loop {
                match listener.accept().await {
                    Ok((mut socket, addr)) => {
                        let key = privkey.clone();
                        let vote_service = vote_service.clone();

                        tokio::spawn(async move {
                            if debug {
                                log::debug!("Accepted vote connection from {}", addr);
                            }

                            let _ = socket.write_all(b"VOTIFIER 1.9\n").await;
                            let result = match VoteHandler::handle_v1(&mut socket, &key).await {
                                Ok(v) => Ok(v),
                                Err(_) => VoteHandler::handle_v2(&mut socket).await,
                            };

                            match result {
                                Ok(vote) => {
                                    log::info!(
                                        "Received vote from {} for service {}",
                                        vote.username, vote.service_name
                                    );

                                    vote_service.emit(Vote {
                                        service_name: vote.service_name,
                                        username: vote.username,
                                        address: vote.address,
                                        timestamp: vote.timestamp,
                                    });
                                }
                                Err(e) => {
                                    log::warn!("Vote error {}: {}", addr, e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Accept error: {}", e);
                    }
                }
            }
        });
    });
    log::info!("VoteMe plugin loaded successfully.");

    Ok(())
}

#[plugin_impl]
pub struct VoteMe;

impl VoteMe {
    pub fn new() -> Self {
        VoteMe
    }
}

impl Default for VoteMe {
    fn default() -> Self {
        Self::new()
    }
}
