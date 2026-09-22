use std::collections::HashMap;
use std::io::Cursor;

use crate::protocol::{self, V2InnerPayload};

#[test]
fn test_v2_signing_and_verification_flow() {
    let mut tokens = HashMap::new();
    tokens.insert("default".to_string(), "MySecretToken123".to_string());

    let challenge = "SecureConnectionChallenge456";

    let inner = V2InnerPayload {
        service_name: "Minecraft-MP".to_string(),
        username: "Alex".to_string(),
        address: "192.168.1.100".to_string(),
        timestamp: "1695385000".to_string(),
        challenge: challenge.to_string(),
    };

    let v2_json = protocol::format_v2_packet(&inner, "MySecretToken123")
        .expect("Failed to format V2 packet");

    let request = protocol::parse_v2_packet(v2_json.as_bytes(), &tokens, challenge)
        .expect("Failed to parse valid V2 packet");

    assert_eq!(request.service_name, "Minecraft-MP");
    assert_eq!(request.username, "Alex");
    assert_eq!(request.address, "192.168.1.100");
}

#[test]
fn test_v2_rejects_corrupted_signature() {
    let mut tokens = HashMap::new();
    tokens.insert("default".to_string(), "MySecretToken123".to_string());

    let challenge = "SecureChallenge";

    let inner = V2InnerPayload {
        service_name: "TopG".to_string(),
        username: "BadActor".to_string(),
        address: "10.0.0.1".to_string(),
        timestamp: "123456".to_string(),
        challenge: challenge.to_string(),
    };

    let v2_json =
        protocol::format_v2_packet(&inner, "WrongToken").expect("Failed to format V2 packet");

    let result = protocol::parse_v2_packet(v2_json.as_bytes(), &tokens, challenge);
    assert!(result.is_err());
}

#[test]
fn test_v2_rejects_challenge_mismatch() {
    let mut tokens = HashMap::new();
    tokens.insert("default".to_string(), "TokenXYZ".to_string());

    let inner = V2InnerPayload {
        service_name: "TopG".to_string(),
        username: "Steve".to_string(),
        address: "10.0.0.1".to_string(),
        timestamp: "123456".to_string(),
        challenge: "StaleChallenge".to_string(),
    };

    let v2_json =
        protocol::format_v2_packet(&inner, "TokenXYZ").expect("Failed to format V2 packet");

    let result = protocol::parse_v2_packet(v2_json.as_bytes(), &tokens, "NewChallenge");
    assert!(result.is_err());
}

#[test]
fn test_proxy_v1_header_processing() {
    let data = b"PROXY TCP4 203.0.113.195 198.51.100.1 56324 8192\r\n{rest_of_packet}";
    let mut reader = Cursor::new(&data[..]);
    let mut writer = Vec::new();

    let result = protocol::process_proxy_headers(&mut reader, &mut writer)
        .expect("Failed to parse PROXY header");

    assert_eq!(result.real_client_ip, Some("203.0.113.195".to_string()));
}
