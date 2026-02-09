use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

pub fn encrypt(data: &[u8], pubkey: &RsaPublicKey) -> Vec<u8> {
    pubkey
        .encrypt(&mut rand::thread_rng(), Pkcs1v15Encrypt, data)
        .expect("RSA encrypt failed")
}

pub fn decrypt(data: &[u8], privkey: &RsaPrivateKey) -> Vec<u8> {
    privkey
        .decrypt(Pkcs1v15Encrypt, data)
        .expect("RSA decrypt failed")
}
