// voteme-api/src/lib.rs
use std::os::raw::c_char;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Vote {
    pub service_name: String,
    pub username: String,
    pub address: String,
    pub timestamp: u64,
}

/// Event yang dikirim ketika vote diterima
#[derive(Clone, Debug)]
pub struct VoteReceivedEvent {
    pub vote: Vote,
    pub received_at: std::time::SystemTime,
}

// Function pointer types (ABI contract)
pub type GetVotesFn = unsafe extern "C" fn(len: *mut usize) -> *const Vote;
pub type FreeVotesFn = unsafe extern "C" fn(ptr: *const Vote, len: usize);
pub type FreeStringFn = unsafe extern "C" fn(ptr: *mut c_char);

// Event handler type
pub type VoteEventHandler = Arc<dyn Fn(VoteReceivedEvent) + Send + Sync>;

/// Helper untuk mendapatkan informasi vote
impl Vote {
    pub fn new(service_name: String, username: String, address: String, timestamp: u64) -> Self {
        Vote {
            service_name,
            username,
            address,
            timestamp,
        }
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl VoteReceivedEvent {
    pub fn vote(&self) -> &Vote {
        &self.vote
    }

    pub fn received_at(&self) -> std::time::SystemTime {
        self.received_at
    }
}

