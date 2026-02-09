use std::sync::Arc;
use tokio::sync::RwLock;
use voteme_api::{Vote, VoteReceivedEvent};

/// Global event manager untuk menangani VoteReceivedEvent
pub struct EventManager {
    handlers: Arc<RwLock<Vec<Arc<dyn Fn(VoteReceivedEvent) + Send + Sync>>>>,
}

impl EventManager {
    pub fn new() -> Self {
        EventManager {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register handler untuk menerima VoteReceivedEvent
    pub async fn subscribe<F>(&self, handler: F)
    where
        F: Fn(VoteReceivedEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().await;
        handlers.push(Arc::new(handler));
        log::debug!("Vote event handler registered. Total handlers: {}", handlers.len());
    }

    /// Emit VoteReceivedEvent ke semua handlers yang telah di-register
    pub async fn emit(&self, vote: Vote) {
        let event = VoteReceivedEvent {
            vote: vote.clone(),
            received_at: std::time::SystemTime::now(),
        };

        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            handler(event.clone());
        }
    }

    /// Dapatkan jumlah handlers yang terdaftar
    pub async fn handler_count(&self) -> usize {
        self.handlers.read().await.len()
    }
}

impl Clone for EventManager {
    fn clone(&self) -> Self {
        EventManager {
            handlers: Arc::clone(&self.handlers),
        }
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}
