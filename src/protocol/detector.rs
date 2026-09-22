use crate::error::{VotifierError, VotifierResult};
use crate::protocol::version::VoteProtocolVersion;

pub const PROTOCOL_2_MAGIC: u16 = 0x733A;

pub fn detect_protocol_version(header: &[u8]) -> VotifierResult<VoteProtocolVersion> {
    if header.is_empty() {
        return Err(VotifierError::ProtocolViolation(
            "Empty header: not enough data to determine protocol version".to_string(),
        ));
    }

    if header[0] == b'{' || header[0] == b'[' {
        return Ok(VoteProtocolVersion::V2);
    }

    if header.len() >= 2 {
        let magic = ((header[0] as u16) << 8) | (header[1] as u16);
        if magic == PROTOCOL_2_MAGIC {
            return Ok(VoteProtocolVersion::V2);
        }
    }

    Ok(VoteProtocolVersion::V1)
}
