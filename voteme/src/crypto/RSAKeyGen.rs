use rand::thread_rng;
use rsa::{RsaPrivateKey, RsaPublicKey};

pub fn generate(bits: usize) -> (RsaPrivateKey, RsaPublicKey) {
    log::warn!("============================");
    log::warn!("Generating {}-bit RSA keypair...", bits);
    log::warn!("============================");

    let mut rng = thread_rng();
    let privkey = RsaPrivateKey::new(&mut rng, bits).expect("Key gen failed");
    let pubkey = RsaPublicKey::from(&privkey);
    (privkey, pubkey)
}
