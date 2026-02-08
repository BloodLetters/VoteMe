use std::path::Path;
use std::sync::Arc;

use pumpkin::plugin::Context;
use pumpkin_api_macros::{plugin_impl, plugin_method};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

mod crypto;
mod file;
mod net;
mod parser;

use crypto::{RSAIO, RSAKeyGen};
use file::{Config, ConfigManager};
use net::vote_handler::VoteHandler;

#[plugin_method]
async fn on_load(&mut self, server: Arc<Context>) -> Result<(), String> {
    server.init_log();
    log::info!("VoteMe plugin loading...");

    // Initialize config
    let mut config = Config::default();
    let mut config_manager = ConfigManager::new_default();
    config_manager.init_config(&mut config).await?;

    log::info!(
        "Config loaded - Host: {}, Port: {}, RSA Bits: {}",
        config.host,
        config.port,
        config.rsa_bits
    );

    // Load or generate RSA keypair
    let privkey = if Path::new("plugins/VoteMe/rsa/private.key").exists() {
        log::info!("Loading existing RSA keypair...");
        RSAIO::load_private("private.key")
    } else {
        log::info!(
            "private.key not found, generating new RSA keypair with {} bits...",
            config.rsa_bits
        );
        let (privkey, pubkey) = RSAKeyGen::generate(config.rsa_bits as usize);
        RSAIO::save_private(&privkey, "private.key");
        RSAIO::save_public(&pubkey, "public.key");
        log::info!("Generated private.key and public.key");
        privkey
    };

    log::info!("Starting VoteMe server on {}:{}...", config.host, config.port);

    // Spawn the vote server in a new runtime task
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let bind_addr = format!("{}:{}", config.host, config.port);
            let listener = match TcpListener::bind(&bind_addr).await {
                Ok(l) => {
                    log::info!("VoteMe listening on {}", bind_addr);
                    l
                }
                Err(e) => {
                    log::error!("Failed to bind to {}: {}", bind_addr, e);
                    return;
                }
            };

            loop {
                match listener.accept().await {
                    Ok((mut socket, addr)) => {
                        let key = privkey.clone();

                        tokio::spawn(async move {
                            if config.debug {
                                log::debug!("Accepted vote connection from {}", addr)
                            }

                            let _ = socket.write_all(b"VOTIFIER 1.9\n").await;

                            // Try with v1 first, then v2 if v1 fails
                            let result = match VoteHandler::handle_v1(&mut socket, &key).await {
                                Ok(vote) => Ok(vote),
                                Err(_) => VoteHandler::handle_v2(&mut socket).await,
                            };

                            match result {
                                Ok(vote) => {
                                    log::info!("New vote received: {:?}", vote);
                                    voteme_api::on_vote_received(vote);
                                }
                                Err(e) => {
                                    log::warn!("Vote handler error from {}: {}", addr, e)
                                }
                            }

                            if config.debug {
                                log::debug!("Vote connection closed: {}", addr)
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Error accepting vote connection: {}", e);
                    }
                }
            }
        });
    });

    log::info!("VoteMe plugin loaded successfully.");
    voteme_api::set_voteme_loaded(true);
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
