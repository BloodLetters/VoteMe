use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, OnceLock,
};

/// Vote event data shared across plugins.
#[derive(Debug, Clone)]
pub struct Vote {
    pub service_name: String,
    pub username: String,
    pub address: String,
    pub timestamp: String,
}

/// Internal "loaded" flag for the VoteMe plugin.
static VOTEME_LOADED: AtomicBool = AtomicBool::new(false);

/// Callback signature for vote events.
///
/// Note: kept as an `Arc<dyn Fn(Vote)>` so it can be safely shared across threads.
pub type VoteReceivedCallback = Arc<dyn Fn(Vote) + Send + Sync + 'static>;

static ON_VOTE_RECEIVED: OnceLock<Mutex<Option<VoteReceivedCallback>>> = OnceLock::new();

fn on_vote_received_store() -> &'static Mutex<Option<VoteReceivedCallback>> {
    ON_VOTE_RECEIVED.get_or_init(|| Mutex::new(None))
}

/// Returns `true` if the VoteMe plugin has finished its load routine.
pub fn voteme_loaded() -> bool {
    VOTEME_LOADED.load(Ordering::Relaxed)
}

/// Sets the internal loaded state.
///
/// Intended to be called by VoteMe itself.
pub fn set_voteme_loaded(loaded: bool) {
    VOTEME_LOADED.store(loaded, Ordering::Relaxed);
}

/// Registers a callback that will be invoked whenever a new vote is received.
///
/// Only a single callback is stored; registering again replaces the previous one.
pub fn set_on_vote_received(callback: VoteReceivedCallback) {
    let mut guard = on_vote_received_store()
        .lock()
        .expect("on_vote_received mutex poisoned");
    *guard = Some(callback);
}

/// Clears the current vote callback (if any).
pub fn clear_on_vote_received() {
    let mut guard = on_vote_received_store()
        .lock()
        .expect("on_vote_received mutex poisoned");
    *guard = None;
}

/// Triggers the vote-received callback (if registered).
///
/// Intended to be called by VoteMe's networking code when a vote arrives.
pub fn on_vote_received(vote: Vote) {
    let callback = {
        let guard = on_vote_received_store()
            .lock()
            .expect("on_vote_received mutex poisoned");
        guard.clone()
    };

    if let Some(cb) = callback {
        cb(vote);
    }
}
