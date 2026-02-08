mod crypto;
mod net;
mod parser;

use crypto::{RSAIO, RSAKeyGen};
use net::vote_handler::VoteHandler;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let privkey = if Path::new("private.key").exists() {
        RSAIO::load_private("private.key")
    } else {
        println!("private.key not found, generating new RSA keypair...");
        let (privkey, pubkey) = RSAKeyGen::generate(2048);
        RSAIO::save_private(&privkey, "private.key");
        RSAIO::save_public(&pubkey, "public.key");
        println!("Generated private.key and public.key");
        privkey
    };

    let listener = TcpListener::bind("0.0.0.0:8192").await.expect("Bind failed");

    println!("LISTENING 0.0.0.0:8192");

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let key = privkey.clone();

        tokio::spawn(async move {
            println!("CONNECTED {}", addr);
            let _ = socket.write_all(b"VOTIFIER 1.9\n").await;

            // Try with v1 first, then v2 if v1 fails
            let result = match VoteHandler::handle_v1(&mut socket, &key).await {
                Ok(vote) => Ok(vote),
                Err(_) => VoteHandler::handle_v2(&mut socket).await,
            };

            match result {
                Ok(vote) => println!("NEW VOTE {:?}", vote),
                Err(e) => println!("VOTIFIER ERROR from {}: {}", addr, e),
            }

            println!("DISCONNECTED {}", addr);
        });
    }
}
