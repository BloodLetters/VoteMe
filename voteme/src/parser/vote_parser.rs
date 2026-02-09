use crate::net::vote_handler::{VoteHandlerError};
use serde::Deserialize;
use voteme_api::Vote;
pub struct VoteParser;

impl VoteParser {
    /// Parse a Votifier v1 by tcp payload (after decryption).
    pub fn parse_v1(plaintext: &str) -> Result<Vote, VoteHandlerError> {
        let mut lines = plaintext.lines();

        let header = lines
            .next()
            .ok_or_else(|| VoteHandlerError::InvalidPacket("Empty payload".to_string()))?;

        if header != "VOTE" {
            return Err(VoteHandlerError::InvalidPacket(format!(
                "Unexpected vote header: {:?}",
                header
            )));
        }

        let service_name = lines
            .next()
            .ok_or_else(|| VoteHandlerError::InvalidPacket("Missing service name".to_string()))?
            .to_string();

        let username = lines
            .next()
            .ok_or_else(|| VoteHandlerError::InvalidPacket("Missing username".to_string()))?
            .to_string();

        let address = lines
            .next()
            .ok_or_else(|| VoteHandlerError::InvalidPacket("Missing address".to_string()))?
            .to_string();

        let timestamp_str = lines
            .next()
            .ok_or_else(|| VoteHandlerError::InvalidPacket("Missing timestamp".to_string()))?
            .to_string();

        let timestamp = timestamp_str.parse::<u64>()
            .map_err(|_| VoteHandlerError::InvalidPacket("Invalid timestamp".to_string()))?;

        Ok(Vote {
            service_name,
            username,
            address,
            timestamp: timestamp.to_string(),
        })
    }

    /// Parse a Votifier v2 JSON payload (after decryption).
    pub fn parse_v2(json: &str) -> Result<Vote, VoteHandlerError> {
        #[derive(Deserialize)]
        struct V2Payload {
            serviceName: String,
            username: String,
            address: String,
            timestamp: i64,
        }

        let payload: V2Payload = serde_json::from_str(json).map_err(|e| {
            VoteHandlerError::InvalidPacket(format!("Invalid JSON: {}", e))
        })?;

        Ok(Vote {
            service_name: payload.serviceName,
            username: payload.username,
            address: payload.address,
            timestamp: payload.timestamp.to_string(),
        })
    }
}
