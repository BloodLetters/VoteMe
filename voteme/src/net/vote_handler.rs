use crate::crypto::RSA;
use crate::parser::vote_parser::VoteParser;

use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::vote::Vote;

#[derive(Debug)]
pub enum VoteHandlerError {
    Io(std::io::Error),
    InvalidUtf8(std::string::FromUtf8Error),
    InvalidPacket(String),
}

impl std::fmt::Display for VoteHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoteHandlerError::Io(e) => write!(f, "io error: {e}"),
            VoteHandlerError::InvalidUtf8(e) => write!(f, "invalid utf8: {e}"),
            VoteHandlerError::InvalidPacket(msg) => write!(f, "invalid packet: {msg}"),
        }
    }
}

impl std::error::Error for VoteHandlerError {}

impl From<std::io::Error> for VoteHandlerError {
    fn from(value: std::io::Error) -> Self {
        VoteHandlerError::Io(value)
    }
}

impl From<std::string::FromUtf8Error> for VoteHandlerError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        VoteHandlerError::InvalidUtf8(value)
    }
}

pub struct VoteHandler;

impl VoteHandler {
    /// Votifier v1
    pub async fn handle_v1(socket: &mut TcpStream, key: &RsaPrivateKey) -> Result<Vote, VoteHandlerError> {
        socket.write_all(b"VOTIFIER 1.9\n").await?;

        let rsa_size = key.size();
        let mut rsa_block = vec![0u8; rsa_size];
        socket.read_exact(&mut rsa_block).await?;

        let mut decrypted = RSA::decrypt(&rsa_block, key);

        if let Some(nul) = decrypted.iter().position(|&b| b == 0) {
            decrypted.truncate(nul);
        }

        let plaintext = String::from_utf8(decrypted)?;

        VoteParser::parse_v1(&plaintext)
    }

    pub async fn handle_v2(socket: &mut TcpStream) -> Result<Vote, VoteHandlerError> {
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > 64 * 1024 {
            return Err(VoteHandlerError::InvalidPacket("v2 payload too large".to_string()));
        }

        let mut buf = vec![0u8; len];
        socket.read_exact(&mut buf).await?;

        let json = String::from_utf8(buf)?;
        VoteParser::parse_v2(&json)
    }
}
