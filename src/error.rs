#[derive(Debug, thiserror::Error)]
pub enum VotifierError {
    #[error("Invalid vote payload: {0}")]
    InvalidPayload(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Client rate-limited ({client_identity}), please retry after {retry_after_seconds}s")]
    RateLimited {
        client_identity: String,
        retry_after_seconds: u64,
    },

    #[error("Client temporarily banned ({client_identity}) due to repeated failures, banned for {remaining_seconds}s")]
    ClientBanned {
        client_identity: String,
        remaining_seconds: u64,
    },

    #[error("Protocol error: {0}")]
    ProtocolViolation(String),

    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),

    #[error("Network I/O error: {0}")]
    NetworkIo(#[from] std::io::Error),

    #[error("JSON serialization or deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

pub type VotifierResult<T> = Result<T, VotifierError>;
