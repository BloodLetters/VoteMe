use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::crypto::rsa::RsaKeyManager;
use crate::error::{VotifierError, VotifierResult};
use crate::model::Vote;
use crate::network::connection::{handle_client_connection, ConnectionContext};
use crate::network::throttle::VoteThrottleService;

pub type VoteCallback = Arc<dyn Fn(Vote) + Send + Sync + 'static>;

pub struct VoteReceiver {
    host: String,
    port: u16,
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl VoteReceiver {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    pub fn start(
        &mut self,
        key_manager: Arc<RsaKeyManager>,
        tokens: Arc<HashMap<String, String>>,
        throttle_service: Arc<VoteThrottleService>,
        disable_v1: bool,
        on_vote: VoteCallback,
    ) -> VotifierResult<()> {
        let bind_addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&bind_addr).map_err(|e| {
            VotifierError::NetworkIo(std::io::Error::new(
                e.kind(),
                format!("Failed to bind Votifier on {bind_addr}: {e}"),
            ))
        })?;

        listener.set_nonblocking(true)?;

        let is_running = Arc::clone(&self.running);
        is_running.store(true, Ordering::SeqCst);

        let context = Arc::new(ConnectionContext {
            key_manager,
            tokens,
            throttle_service,
            disable_v1,
        });

        let handle = thread::Builder::new()
            .name("votifier-io-listener".to_string())
            .spawn(move || {
                while is_running.load(Ordering::SeqCst) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let worker_context = Arc::clone(&context);
                            let worker_callback = Arc::clone(&on_vote);

                            thread::Builder::new()
                                .name("votifier-worker".to_string())
                                .spawn(move || {
                                    match handle_client_connection(stream, &worker_context) {
                                        Ok(vote) => {
                                            worker_callback(vote);
                                        }
                                        Err(err) => {
                                            let _ = err;
                                        }
                                    }
                                })
                                .ok();
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(_) => {
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
            })
            .map_err(VotifierError::NetworkIo)?;

        self.thread_handle = Some(handle);
        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn host(&self) -> &str {
        &self.host
    }
}

impl Drop for VoteReceiver {
    fn drop(&mut self) {
        self.shutdown();
    }
}
