use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub const DEFAULT_CHALLENGE_LENGTH: usize = 24;

pub fn generate_secure_challenge() -> String {
    generate_secure_challenge_with_length(DEFAULT_CHALLENGE_LENGTH)
}

pub fn generate_secure_challenge_with_length(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn verify_challenge(received: &str, expected: &str) -> bool {
    received.trim() == expected.trim()
}
