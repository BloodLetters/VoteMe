use aes::Aes128;
use cbc::{Decryptor, Encryptor};
use cipher::{
    block_padding::Pkcs7,
    BlockDecryptMut, BlockEncryptMut, KeyIvInit
};

pub fn decrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut buf = data.to_vec();

    let decrypted = Decryptor::<Aes128>::new(key.into(), iv.into())
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .expect("AES decrypt failed");

    decrypted.to_vec()
}

pub fn encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut buf = data.to_vec();

    let encrypted = Encryptor::<Aes128>::new(key.into(), iv.into())
        .encrypt_padded_mut::<Pkcs7>(&mut buf, data.len())
        .expect("AES encrypt failed");

    encrypted.to_vec()
}
