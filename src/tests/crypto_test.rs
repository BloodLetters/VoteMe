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

#[test]
fn test_rsa_spki_and_raw_key_files() {
    let key_mgr = RsaKeyManager::generate_new_keys().expect("Failed to generate RSA keys");
    let temp_dir = std::env::temp_dir().join(format!(
        "voteme_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let pub_key_file = temp_dir.join("public.key");
    let priv_key_file = temp_dir.join("private.key");

    key_mgr
        .save_raw_key_files(&priv_key_file, &pub_key_file)
        .unwrap();

    let pub_key_content = std::fs::read_to_string(&pub_key_file).unwrap();
    assert!(pub_key_content.starts_with("MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A"));

    let loaded_mgr =
        RsaKeyManager::load_from_raw_key_files(&priv_key_file, &pub_key_file).unwrap();

    let plaintext = b"VOTE\nPlanetMinecraft\nSteve\n127.0.0.1\n1695384000\n";
    let encrypted = loaded_mgr.encrypt_v1_data(plaintext).unwrap();
    let decrypted = key_mgr.decrypt_v1_block(&encrypted).unwrap();
    assert_eq!(&decrypted, plaintext);

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_v1_empty_service_name_and_blank_lines() {
    let key_mgr = RsaKeyManager::generate_new_keys().expect("Failed to generate RSA keys");

    let empty_service_payload = b"VOTE\n\nSteve\n127.0.0.1\n1695384000\n";
    let encrypted = key_mgr.encrypt_v1_data(empty_service_payload).unwrap();
    let request = protocol::parse_v1_packet(&encrypted, &key_mgr).unwrap();
    assert_eq!(request.service_name, "Votifier");
    assert_eq!(request.username, "Steve");
    assert_eq!(request.address, "127.0.0.1");
    assert_eq!(request.timestamp, "1695384000");

    let blank_lines_payload = b"VOTE\r\n\r\nPlanetMinecraft\r\nSteve\r\n127.0.0.1\r\n1695384000\r\n";
    let encrypted_blank = key_mgr.encrypt_v1_data(blank_lines_payload).unwrap();
    let request_blank = protocol::parse_v1_packet(&encrypted_blank, &key_mgr).unwrap();
    assert_eq!(request_blank.service_name, "PlanetMinecraft");
    assert_eq!(request_blank.username, "Steve");
}
