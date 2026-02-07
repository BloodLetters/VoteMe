use rsa::{RsaPrivateKey, RsaPublicKey};
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
