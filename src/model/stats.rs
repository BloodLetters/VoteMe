use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default)]
pub struct VoteStatistics {
    total_received: AtomicU64,
    valid_votes: AtomicU64,
    failed_votes: AtomicU64,
    v1_votes: AtomicU64,
    v2_votes: AtomicU64,
    last_vote_epoch_seconds: AtomicU64,
}

impl VoteStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_incoming_connection(&self) {
        self.total_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_successful_vote(&self, is_v2: bool) {
        self.valid_votes.fetch_add(1, Ordering::Relaxed);
        if is_v2 {
            self.v2_votes.fetch_add(1, Ordering::Relaxed);
        } else {
            self.v1_votes.fetch_add(1, Ordering::Relaxed);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_vote_epoch_seconds.store(now, Ordering::Relaxed);
    }

    pub fn record_failed_vote(&self) {
        self.failed_votes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn total_received(&self) -> u64 {
        self.total_received.load(Ordering::Relaxed)
    }

    pub fn valid_votes(&self) -> u64 {
        self.valid_votes.load(Ordering::Relaxed)
    }

    pub fn failed_votes(&self) -> u64 {
        self.failed_votes.load(Ordering::Relaxed)
    }

    pub fn v1_votes(&self) -> u64 {
        self.v1_votes.load(Ordering::Relaxed)
    }

    pub fn v2_votes(&self) -> u64 {
        self.v2_votes.load(Ordering::Relaxed)
    }

    pub fn last_vote_epoch_seconds(&self) -> u64 {
        self.last_vote_epoch_seconds.load(Ordering::Relaxed)
    }
}
