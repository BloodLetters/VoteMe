use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePrivateKey, DecodePublicKey};
use std::fs;

pub fn save_private(key: &RsaPrivateKey, path: &str) {
    let pem = key.to_pkcs8_pem(Default::default()).unwrap();
    fs::write(path, pem).unwrap();
}

pub fn save_public(key: &RsaPublicKey, path: &str) {
    let pem = key.to_public_key_pem(Default::default()).unwrap();
    fs::write(path, pem).unwrap();
}

pub fn load_private(path: &str) -> RsaPrivateKey {
    let pem = fs::read_to_string(path).unwrap();
    RsaPrivateKey::from_pkcs8_pem(&pem).unwrap()
}

pub fn load_public(path: &str) -> RsaPublicKey {
    let pem = fs::read_to_string(path).unwrap();
    RsaPublicKey::from_public_key_pem(&pem).unwrap()
}
