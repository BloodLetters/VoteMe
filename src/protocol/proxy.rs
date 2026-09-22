use std::io::{BufRead, Write};

use crate::error::{VotifierError, VotifierResult};

pub const MAX_PROXY_V1_LINE_LENGTH: usize = 108;
pub const MAX_HTTP_CONNECT_HEADERS: usize = 64;

#[derive(Debug, Default, Clone)]
pub struct ProxyHeaderResult {
    pub real_client_ip: Option<String>,
}

pub fn process_proxy_headers<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> VotifierResult<ProxyHeaderResult> {
    let mut result = ProxyHeaderResult::default();

    let peek_bytes = reader.fill_buf()?;
    if peek_bytes.is_empty() {
        return Ok(result);
    }

    if peek_bytes.starts_with(b"PROXY ") {
        let mut line = String::new();
        reader.read_line(&mut line)?;

        if line.len() > MAX_PROXY_V1_LINE_LENGTH {
            return Err(VotifierError::ProtocolViolation(
                "PROXY v1 line exceeds maximum allowed length".to_string(),
            ));
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let src_ip = parts[2].trim();
            if !src_ip.is_empty() && src_ip != "UNKNOWN" {
                result.real_client_ip = Some(src_ip.to_string());
            }
        }
        return Ok(result);
    }

    if peek_bytes.starts_with(b"CONNECT ") {
        let mut first_line = String::new();
        reader.read_line(&mut first_line)?;

        let mut header_count = 0;
        loop {
            let mut header_line = String::new();
            let bytes_read = reader.read_line(&mut header_line)?;
            if bytes_read == 0 || header_line.trim().is_empty() {
                break;
            }
            header_count += 1;
            if header_count > MAX_HTTP_CONNECT_HEADERS {
                return Err(VotifierError::ProtocolViolation(
                    "Too many HTTP CONNECT headers received".to_string(),
                ));
            }
        }

        writer.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;
        writer.flush()?;
        return Ok(result);
    }

    Ok(result)
}
