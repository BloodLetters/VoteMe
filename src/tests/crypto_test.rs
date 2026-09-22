use crate::crypto::{self, RsaKeyManager};
use crate::protocol;

#[test]
fn test_v1_encryption_and_decryption_flow() {
    let key_mgr = RsaKeyManager::generate_new_keys().expect("Failed to generate RSA keys");

    let v1_payload =
        protocol::format_v1_payload("PlanetMinecraft", "Steve", "127.0.0.1", "1695384000");

    let encrypted_block = key_mgr
        .encrypt_v1_data(&v1_payload)
        .expect("Failed to encrypt V1 block");

    assert_eq!(encrypted_block.len(), crypto::V1_BLOCK_BYTES);

    let request = protocol::parse_v1_packet(&encrypted_block, &key_mgr)
        .expect("Failed to parse V1 packet");

    assert_eq!(request.service_name, "PlanetMinecraft");
    assert_eq!(request.username, "Steve");
    assert_eq!(request.address, "127.0.0.1");
    assert_eq!(request.timestamp, "1695384000");
}
