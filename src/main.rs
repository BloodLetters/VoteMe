mod crypto;

use tokio::{net::TcpListener, io::AsyncReadExt};
use crypto::{RSAIO, RSA, AES};
use rsa::RsaPrivateKey;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Vote {
    serviceName: String,
    username: String,
    address: String,
    timestamp: String,
}

#[tokio::main]
async fn main() {
    let privkey = RSAIO::load_private("private.key");

    let listener = TcpListener::bind("0.0.0.0:8192")
        .await
        .expect("Bind failed");

    println!("LISTENING 0.0.0.0:8192");

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let key = privkey.clone();

        tokio::spawn(async move {
            println!("CONNECTED {}", addr);

            let mut buffer = Vec::new();
            let mut temp = [0u8; 1024];

            loop {
                let n = match socket.read(&mut temp).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => break,
                };

                buffer.extend_from_slice(&temp[..n]);

                match try_parse_packet(&buffer, &key) {
                    Ok(Some(vote)) => {
                        println!("NEW VOTE {:?}", vote);
                        break;
                    }
                    Ok(None) => {
                        // belum cukup data, lanjut baca
                        continue;
                    }
                    Err(e) => {
                        println!("INVALID PACKET: {}", e);
                        break;
                    }
                }
            }
        });
    }
}

fn try_parse_packet(data: &[u8], key: &RsaPrivateKey) -> Result<Option<Vote>, String> {
    if data.len() < 2 {
        return Ok(None);
    }

    let rsa_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    let total_min = 2 + rsa_len + 16;

    if data.len() < total_min {
        return Ok(None); // belum lengkap
    }

    let rsa_block = &data[2..2 + rsa_len];
    let iv = &data[2 + rsa_len..2 + rsa_len + 16];
    let payload = &data[2 + rsa_len + 16..];

    let aes_key = RSA::decrypt(rsa_block, key);
    let decrypted = AES::decrypt(payload, &aes_key, iv);

    let json = String::from_utf8(decrypted)
        .map_err(|_| "Invalid UTF8")?;

    let vote: Vote = serde_json::from_str(&json)
        .map_err(|_| "Invalid JSON")?;

    Ok(Some(vote))
}
