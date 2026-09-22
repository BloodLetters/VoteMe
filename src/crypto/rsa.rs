use std::fs;
use std::path::Path;

use base64::Engine;
use pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey};
use pkcs8::{
    DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding,
};
use rand::rngs::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

use crate::error::{VotifierError, VotifierResult};

pub const V1_BLOCK_BYTES: usize = 256;

pub struct RsaKeyManager {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl RsaKeyManager {
    pub fn generate_new_keys() -> VotifierResult<Self> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048)
            .map_err(|e| VotifierError::CryptoError(format!("Failed to generate RSA key: {e}")))?;
        let public_key = RsaPublicKey::from(&private_key);

        Ok(Self {
            private_key,
            public_key,
        })
    }

    pub fn load_or_generate_in_directory(directory: &Path) -> VotifierResult<Self> {
        if !directory.exists() {
            fs::create_dir_all(directory).map_err(|e| {
                VotifierError::ConfigurationError(format!(
                    "Cannot create RSA directory at {}: {e}",
                    directory.display()
                ))
            })?;
        }

        let private_key_file = directory.join("private.pem");
        let public_key_file = directory.join("public.pem");
        let private_raw_key = directory.join("private.key");
        let public_raw_key = directory.join("public.key");

        if private_key_file.exists() && public_key_file.exists() {
            let manager = Self::load_from_files(&private_key_file, &public_key_file)?;
            if !public_raw_key.exists() || !private_raw_key.exists() {
                let _ = manager.save_raw_key_files(&private_raw_key, &public_raw_key);
            }
            Ok(manager)
        } else if private_raw_key.exists() && public_raw_key.exists() {
            let manager = Self::load_from_raw_key_files(&private_raw_key, &public_raw_key)?;
            let _ = manager.save_to_files(&private_key_file, &public_key_file);
            Ok(manager)
        } else {
            let manager = Self::generate_new_keys()?;
            manager.save_to_files(&private_key_file, &public_key_file)?;
            manager.save_raw_key_files(&private_raw_key, &public_raw_key)?;
            Ok(manager)
        }
    }

    pub fn load_from_files(private_path: &Path, public_path: &Path) -> VotifierResult<Self> {
        let private_pem = fs::read_to_string(private_path).map_err(|e| {
            VotifierError::CryptoError(format!(
                "Failed to read private key from {}: {e}",
                private_path.display()
            ))
        })?;

        let public_pem = fs::read_to_string(public_path).map_err(|e| {
            VotifierError::CryptoError(format!(
                "Failed to read public key from {}: {e}",
                public_path.display()
            ))
        })?;

        let private_key = RsaPrivateKey::from_pkcs1_pem(&private_pem)
            .or_else(|_| RsaPrivateKey::from_pkcs8_pem(&private_pem))
            .map_err(|e| VotifierError::CryptoError(format!("Invalid private key PEM: {e}")))?;

        let public_key = RsaPublicKey::from_public_key_pem(&public_pem)
            .or_else(|_| RsaPublicKey::from_pkcs1_pem(&public_pem))
            .map_err(|e| VotifierError::CryptoError(format!("Invalid public key PEM: {e}")))?;

        Ok(Self {
            private_key,
            public_key,
        })
    }

    pub fn load_from_raw_key_files(private_path: &Path, public_path: &Path) -> VotifierResult<Self> {
        let private_raw = fs::read_to_string(private_path).map_err(|e| {
            VotifierError::CryptoError(format!(
                "Failed to read private key from {}: {e}",
                private_path.display()
            ))
        })?;

        let public_raw = fs::read_to_string(public_path).map_err(|e| {
            VotifierError::CryptoError(format!(
                "Failed to read public key from {}: {e}",
                public_path.display()
            ))
        })?;

        let private_bytes = base64::prelude::BASE64_STANDARD
            .decode(private_raw.trim())
            .map_err(|e| VotifierError::CryptoError(format!("Invalid Base64 in private key: {e}")))?;

        let public_bytes = base64::prelude::BASE64_STANDARD
            .decode(public_raw.trim())
            .map_err(|e| VotifierError::CryptoError(format!("Invalid Base64 in public key: {e}")))?;

        let private_key = RsaPrivateKey::from_pkcs8_der(&private_bytes)
            .map_err(|e| VotifierError::CryptoError(format!("Invalid PKCS8 private key DER: {e}")))?;

        let public_key = RsaPublicKey::from_public_key_der(&public_bytes)
            .map_err(|e| VotifierError::CryptoError(format!("Invalid SPKI public key DER: {e}")))?;

        Ok(Self {
            private_key,
            public_key,
        })
    }

    pub fn save_to_files(&self, private_path: &Path, public_path: &Path) -> VotifierResult<()> {
        let private_pem = self
            .private_key
            .to_pkcs1_pem(LineEnding::LF)
            .map_err(|e| VotifierError::CryptoError(format!("Cannot encode private key: {e}")))?;

        let public_pem = self
            .public_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| VotifierError::CryptoError(format!("Cannot encode public key: {e}")))?;

        fs::write(private_path, private_pem.as_bytes()).map_err(|e| {
            VotifierError::ConfigurationError(format!(
                "Failed to write private key to {}: {e}",
                private_path.display()
            ))
        })?;

        fs::write(public_path, public_pem.as_bytes()).map_err(|e| {
            VotifierError::ConfigurationError(format!(
                "Failed to write public key to {}: {e}",
                public_path.display()
            ))
        })?;

        Ok(())
    }

    pub fn save_raw_key_files(&self, private_path: &Path, public_path: &Path) -> VotifierResult<()> {
        let spki_der = self
            .public_key
            .to_public_key_der()
            .map_err(|e| VotifierError::CryptoError(format!("Cannot encode SPKI public key: {e}")))?;
        let spki_b64 = base64::prelude::BASE64_STANDARD.encode(spki_der.as_bytes());

        let pkcs8_der = self
            .private_key
            .to_pkcs8_der()
            .map_err(|e| VotifierError::CryptoError(format!("Cannot encode PKCS8 private key: {e}")))?;
        let pkcs8_b64 = base64::prelude::BASE64_STANDARD.encode(pkcs8_der.as_bytes());

        fs::write(public_path, spki_b64.as_bytes()).map_err(|e| {
            VotifierError::ConfigurationError(format!(
                "Failed to write public key to {}: {e}",
                public_path.display()
            ))
        })?;

        fs::write(private_path, pkcs8_b64.as_bytes()).map_err(|e| {
            VotifierError::ConfigurationError(format!(
                "Failed to write private key to {}: {e}",
                private_path.display()
            ))
        })?;

        Ok(())
    }

    pub fn decrypt_v1_block(&self, encrypted_block: &[u8]) -> VotifierResult<Vec<u8>> {
        if encrypted_block.len() != V1_BLOCK_BYTES {
            return Err(VotifierError::InvalidPayload(format!(
                "Expected RSA block length {} bytes, got {}",
                V1_BLOCK_BYTES,
                encrypted_block.len()
            )));
        }

        self.private_key
            .decrypt(Pkcs1v15Encrypt, encrypted_block)
            .map_err(|e| VotifierError::CryptoError(format!("RSA PKCS1v15 decryption failed: {e}")))
    }

    pub fn encrypt_v1_data(&self, plaintext: &[u8]) -> VotifierResult<Vec<u8>> {
        let mut rng = OsRng;
        self.public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, plaintext)
            .map_err(|e| VotifierError::CryptoError(format!("RSA PKCS1v15 encryption failed: {e}")))
    }

    pub fn public_key(&self) -> &RsaPublicKey {
        &self.public_key
    }
}
