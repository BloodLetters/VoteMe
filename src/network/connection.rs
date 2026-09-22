use std::collections::HashMap;
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use crate::crypto::challenge::generate_secure_challenge;
use crate::crypto::rsa::{RsaKeyManager, V1_BLOCK_BYTES};
use crate::error::{VotifierError, VotifierResult};
use crate::model::Vote;
use crate::network::throttle::VoteThrottleService;
use crate::protocol::detector::detect_protocol_version;
use crate::protocol::proxy::process_proxy_headers;
use crate::protocol::v1::parse_v1_packet;
use crate::protocol::v2::parse_v2_packet;
use crate::protocol::version::VoteProtocolVersion;

pub const DEFAULT_SOCKET_TIMEOUT: Duration = Duration::from_secs(5);
pub const MAX_V2_PACKET_LENGTH: usize = 4 + 65535;

pub struct ConnectionContext {
    pub key_manager: Arc<RsaKeyManager>,
    pub tokens: Arc<HashMap<String, String>>,
    pub throttle_service: Arc<VoteThrottleService>,
    pub disable_v1: bool,
}

pub fn handle_client_connection(
    stream: TcpStream,
    context: &ConnectionContext,
) -> VotifierResult<Vote> {
    stream.set_read_timeout(Some(DEFAULT_SOCKET_TIMEOUT))?;
    stream.set_write_timeout(Some(DEFAULT_SOCKET_TIMEOUT))?;

    let peer_addr = stream.peer_addr().map_err(VotifierError::NetworkIo)?;
    let mut client_ip = peer_addr.ip().to_string();

    if let Err(retry_seconds) = context.throttle_service.check_client_allowed(&client_ip) {
        return Err(VotifierError::RateLimited {
            client_identity: client_ip,
            retry_after_seconds: retry_seconds,
        });
    }

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream.try_clone()?;

    let proxy_result = process_proxy_headers(&mut reader, &mut writer)?;
    if let Some(real_ip) = proxy_result.real_client_ip {
        client_ip = real_ip;
        if let Err(retry_seconds) = context.throttle_service.check_client_allowed(&client_ip) {
            return Err(VotifierError::RateLimited {
                client_identity: client_ip,
                retry_after_seconds: retry_seconds,
            });
        }
    }

    let challenge = generate_secure_challenge();

    let greeting = format!("VOTIFIER 2 {challenge}\r\n");
    writer.write_all(greeting.as_bytes())?;
    writer.flush()?;

    let vote = match read_and_parse_packet(&mut reader, context, &challenge, &client_ip) {
        Ok(v) => {
            context.throttle_service.record_success(&client_ip);
            let _ = writer.write_all(b"{\"status\":\"ok\"}\r\n");
            let _ = writer.flush();
            v
        }
        Err(err) => {
            context.throttle_service.record_failure(&client_ip);
            return Err(err);
        }
    };

    Ok(vote)
}

fn read_and_parse_packet<R: Read>(
    reader: &mut R,
    context: &ConnectionContext,
    expected_challenge: &str,
    source_ip: &str,
) -> VotifierResult<Vote> {
    let mut prefix = [0u8; 2];
    reader.read_exact(&mut prefix)?;

    let detected_version = detect_protocol_version(&prefix)?;

    match detected_version {
        VoteProtocolVersion::V1 => {
            if context.disable_v1 {
                return Err(VotifierError::AuthenticationFailed(
                    "Votifier V1 protocol is disabled by configuration".to_string(),
                ));
            }

            let mut block = vec![0u8; V1_BLOCK_BYTES];
            block[0] = prefix[0];
            block[1] = prefix[1];
            reader.read_exact(&mut block[2..])?;

            let request = parse_v1_packet(&block, &context.key_manager)?;
            Ok(request.into_vote(source_ip))
        }
        VoteProtocolVersion::V2 => {
            let mut payload_bytes = Vec::new();

            if ((prefix[0] as u16) << 8 | prefix[1] as u16) == crate::protocol::PROTOCOL_2_MAGIC {
                let mut len_bytes = [0u8; 2];
                reader.read_exact(&mut len_bytes)?;
                let length = ((len_bytes[0] as usize) << 8) | (len_bytes[1] as usize);

                if length > MAX_V2_PACKET_LENGTH {
                    return Err(VotifierError::InvalidPayload(format!(
                        "Framed payload length {length} exceeds maximum"
                    )));
                }

                payload_bytes.resize(length, 0);
                reader.read_exact(&mut payload_bytes)?;
            } else {
                payload_bytes.push(prefix[0]);
                payload_bytes.push(prefix[1]);

                let mut chunk = [0u8; 1024];
                while payload_bytes.len() < MAX_V2_PACKET_LENGTH {
                    let count = reader.read(&mut chunk)?;
                    if count == 0 {
                        break;
                    }
                    payload_bytes.extend_from_slice(&chunk[..count]);
                    if chunk[..count].contains(&b'}') {
                        if serde_json::from_slice::<serde_json::Value>(&payload_bytes).is_ok() {
                            break;
                        }
                    }
                }
            }

            let request = parse_v2_packet(&payload_bytes, &context.tokens, expected_challenge)?;
            Ok(request.into_vote(source_ip))
        }
    }
}
