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
    let raw_lines: Vec<&str> = text.split('\n').map(str::trim).collect();

    let opcode = raw_lines
        .first()
        .copied()
        .ok_or_else(|| VotifierError::InvalidPayload("Missing opcode in V1 block".to_string()))?;

    if opcode != V1_OPCODE_VOTE {
        return Err(VotifierError::InvalidPayload(format!(
            "Expected opcode '{V1_OPCODE_VOTE}', got '{opcode}'"
        )));
    }

    let non_empty: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let (service_name, username, address, timestamp) = if non_empty.len() >= 5 {
        (non_empty[1], non_empty[2], non_empty[3], non_empty[4])
    } else if raw_lines.len() >= 3 && !raw_lines[2].is_empty() {
        let service = if raw_lines[1].is_empty() {
            "Votifier"
        } else {
            raw_lines[1]
        };
        let user = raw_lines[2];
        let addr = raw_lines.get(3).copied().unwrap_or("");
        let ts = raw_lines.get(4).copied().unwrap_or("");
        (service, user, addr, ts)
    } else {
        match non_empty.len() {
            0 | 1 => {
                return Err(VotifierError::InvalidPayload(
                    "Missing username in V1 block".to_string(),
                ));
            }
            2 => ("Votifier", non_empty[1], "", ""),
            3 => ("Votifier", non_empty[1], non_empty[2], ""),
            4 => ("Votifier", non_empty[1], non_empty[2], non_empty[3]),
            _ => (non_empty[1], non_empty[2], non_empty[3], non_empty[4]),
        }
    };

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
