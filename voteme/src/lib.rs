use std::path::Path;
use std::sync::Arc;

use pumpkin::plugin::Context;
use pumpkin_api_macros::{plugin_impl, plugin_method};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use voteme_api::VoteReceivedEvent;

mod crypto;
mod file;
mod net;
mod parser;

use crypto::{RSAIO, RSAKeyGen};
use file::{Config, ConfigManager};
use net::{vote_handler::VoteHandler, EventManager};

/// Global event manager instance untuk VoteReceivedEvent
pub static EVENT_MANAGER: once_cell::sync::Lazy<EventManager> =
    once_cell::sync::Lazy::new(EventManager::new);

/// Akses global event manager
pub fn get_event_manager() -> &'static EventManager {
    &EVENT_MANAGER
}

/// Register handler untuk menerima VoteReceivedEvent
/// 
/// # Example
/// ```
/// voteme::on_vote_received(|event| {
///     println!("Vote received from: {}", event.vote().username());
/// }).await;
/// ```
pub async fn on_vote_received<F>(handler: F)
where
    F: Fn(VoteReceivedEvent) + Send + Sync + 'static,
{
    get_event_manager().subscribe(handler).await;
}



#[plugin_method]
async fn on_load(&mut self, server: Arc<Context>) -> Result<(), String> {
    server.init_log();
    log::info!("VoteMe plugin loading...");

    let mut config = Config::default();
    let mut config_manager = ConfigManager::new_default();
    config_manager.init_config(&mut config).await?;

    log::info!(
        "Config loaded - Host: {:?}, Port: {:?}, RSA Bits: {:?}",
        config.host,
        config.port,
        config.rsa_bits
    );

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
                        let debug = config.debug;

                        tokio::spawn(async move {
                            if debug {
                                log::debug!("Accepted vote connection from {}", addr)
                            }

                            let _ = socket.write_all(b"VOTIFIER 1.9\n").await;

                            let result = match VoteHandler::handle_v1(&mut socket, &key).await {
                                Ok(vote) => Ok(vote),
                                Err(_) => VoteHandler::handle_v2(&mut socket).await,
                            };

                            match result {
                                Ok(vote) => {
                                    log::info!("New vote received from: {}", vote.username);
                                    
                                    // Emit VoteReceivedEvent ke semua registered handlers
                                    let event_manager = get_event_manager();
                                    event_manager.emit(vote).await;
                                }
                                Err(e) => {
                                    log::warn!("Vote handler error from {}: {}", addr, e)
                                }
                            }

                            if debug {
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
