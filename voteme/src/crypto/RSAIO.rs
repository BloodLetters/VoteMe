use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::fs;
use std::path::Path;

const RSA_DIR: &str = "plugins/VoteMe/rsa";

fn ensure_rsa_dir() {
    if !Path::new(RSA_DIR).exists() {
        fs::create_dir_all(RSA_DIR).expect("Failed to create rsa directory");
    }
}

pub fn save_private(key: &RsaPrivateKey, filename: &str) {
    ensure_rsa_dir();
    let full_path = Path::new(RSA_DIR).join(filename);
    let pem = key.to_pkcs8_pem(Default::default()).unwrap();
    fs::write(&full_path, pem).unwrap();
    log::info!("Saved private key to: {}", full_path.display());
}

pub fn save_public(key: &RsaPublicKey, filename: &str) {
    ensure_rsa_dir();
    let full_path = Path::new(RSA_DIR).join(filename);
    let pem = key.to_public_key_pem(Default::default()).unwrap();
    fs::write(&full_path, pem).unwrap();
    log::info!("Saved public key to: {}", full_path.display());
}

pub fn load_private(filename: &str) -> RsaPrivateKey {
    let full_path = Path::new(RSA_DIR).join(filename);
    let pem = fs::read_to_string(&full_path).unwrap();
    log::info!("Loaded private key from: {}", full_path.display());
    RsaPrivateKey::from_pkcs8_pem(&pem).unwrap()
}

pub fn load_public(filename: &str) -> RsaPublicKey {
    let full_path = Path::new(RSA_DIR).join(filename);
    let pem = fs::read_to_string(&full_path).unwrap();
    log::info!("Loaded public key from: {}", full_path.display());
    RsaPublicKey::from_public_key_pem(&pem).unwrap()
}
