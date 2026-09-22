use serde::{Deserialize, Serialize};

pub const MAX_MINECRAFT_USERNAME_LENGTH: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    pub service_name: String,
    pub username: String,
    pub address: String,
    pub timestamp: String,
    pub source_address: String,
}

impl Vote {
    pub fn new(
        service_name: impl Into<String>,
        username: impl Into<String>,
        address: impl Into<String>,
        timestamp: impl Into<String>,
        source_address: impl Into<String>,
    ) -> Self {
        let raw_username = username.into();
        let sanitized_username = if raw_username.len() > MAX_MINECRAFT_USERNAME_LENGTH {
            raw_username.chars().take(MAX_MINECRAFT_USERNAME_LENGTH).collect()
        } else {
            raw_username
        };

        Self {
            service_name: service_name.into(),
            username: sanitized_username,
            address: address.into(),
            timestamp: timestamp.into(),
            source_address: source_address.into(),
        }
    }

    pub fn is_test_vote(&self) -> bool {
        self.timestamp.eq_ignore_ascii_case("TestVote")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteRequest {
    pub service_name: String,
    pub username: String,
    pub address: String,
    pub timestamp: String,
}

impl VoteRequest {
    pub fn into_vote(self, source_address: impl Into<String>) -> Vote {
        Vote::new(
            self.service_name,
            self.username,
            self.address,
            self.timestamp,
            source_address,
        )
    }
}
