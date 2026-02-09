use rsa::{RsaPrivateKey, RsaPublicKey};
use crate::crypto::{RSAIO, RSAKeyGen};
use rand::thread_rng;

pub fn generate(bits: usize) -> (RsaPrivateKey, RsaPublicKey) {
    log::warn!("============================");
    log::warn!("Generating {}-bit RSA keypair...", bits);
    log::warn!("============================");

    let mut rng = thread_rng();
    let privkey = RsaPrivateKey::new(&mut rng, bits)
        .expect("Key gen failed");
    let pubkey = RsaPublicKey::from(&privkey);
    (privkey, pubkey)
}

pub fn rsa_generate() -> RsaPrivateKey {
    // Keep filenames simple and stable.
    let filename = "private.pem";

    let key_path = std::path::Path::new("plugins/VoteMe/rsa").join(filename);
    if key_path.exists() {
        return RSAIO::load_private(filename);
    }

    let (privkey, pubkey) = RSAKeyGen::generate(2048);
    RSAIO::save_private(&privkey, filename);
    RSAIO::save_public(&pubkey, "public.pem");
    privkey
}
