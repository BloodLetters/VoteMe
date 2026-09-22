use hmac::{Hmac, Mac};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::error::{VotifierError, VotifierResult};

type HmacSha256 = Hmac<Sha256>;

pub fn generate_service_token() -> String {
    generate_service_token_with_length(16)
}

pub fn generate_service_token_with_length(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn compute_hmac_sha256(payload: &[u8], secret_key: &[u8]) -> VotifierResult<Vec<u8>> {
    let mut mac = HmacSha256::new_from_slice(secret_key)
        .map_err(|e| VotifierError::CryptoError(format!("Invalid HMAC key length: {e}")))?;
    mac.update(payload);
    Ok(mac.finalize().into_bytes().to_vec())
}

pub fn verify_hmac_signature(
    provided_signature: &[u8],
    payload: &[u8],
    secret_key: &[u8],
) -> VotifierResult<bool> {
    let expected_signature = compute_hmac_sha256(payload, secret_key)?;

    if provided_signature.len() != expected_signature.len() {
        return Ok(false);
    }

    let is_equal = provided_signature.ct_eq(&expected_signature);
    Ok(is_equal.into())
}
