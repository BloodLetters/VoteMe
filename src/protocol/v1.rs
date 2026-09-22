use crate::crypto::rsa::RsaKeyManager;
use crate::error::{VotifierError, VotifierResult};
use crate::model::VoteRequest;

pub const V1_OPCODE_VOTE: &str = "VOTE";

pub fn parse_v1_packet(
    encrypted_block: &[u8],
    key_manager: &RsaKeyManager,
) -> VotifierResult<VoteRequest> {
    let decrypted = key_manager.decrypt_v1_block(encrypted_block)?;
    parse_v1_decrypted_payload(&decrypted)
}

pub fn parse_v1_decrypted_payload(payload: &[u8]) -> VotifierResult<VoteRequest> {
    let text = String::from_utf8_lossy(payload);
    let mut lines = text.split('\n');

    let opcode = lines
        .next()
        .map(str::trim)
        .ok_or_else(|| VotifierError::InvalidPayload("Missing opcode in V1 block".to_string()))?;

    if opcode != V1_OPCODE_VOTE {
        return Err(VotifierError::InvalidPayload(format!(
            "Expected opcode '{V1_OPCODE_VOTE}', got '{opcode}'"
        )));
    }

    let service_name = lines
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| VotifierError::InvalidPayload("Missing serviceName in V1 block".to_string()))?;

    let username = lines
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| VotifierError::InvalidPayload("Missing username in V1 block".to_string()))?;

    let address = lines.next().map(str::trim).unwrap_or("");
    let timestamp = lines.next().map(str::trim).unwrap_or("");

    Ok(VoteRequest {
        service_name: service_name.to_string(),
        username: username.to_string(),
        address: address.to_string(),
        timestamp: timestamp.to_string(),
    })
}

pub fn format_v1_payload(
    service_name: &str,
    username: &str,
    address: &str,
    timestamp: &str,
) -> Vec<u8> {
    format!("{V1_OPCODE_VOTE}\n{service_name}\n{username}\n{address}\n{timestamp}\n")
        .into_bytes()
}
