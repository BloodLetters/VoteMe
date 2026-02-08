// Example: Cara menggunakan VoteReceivedEvent dari plugin lain

/*
Untuk menggunakan VoteReceivedEvent dari plugin lain yang bergantung pada voteme, 
ikuti langkah-langkah berikut:

1. Tambahkan voteme ke Cargo.toml plugin Anda:
   [dependencies]
   voteme = { path = "../voteme" }

2. Gunakan event handler dalam plugin Anda:
   
   #[plugin_method]
   async fn on_load(&mut self, server: Arc<Context>) -> Result<(), String> {
       // Subscribe ke VoteReceivedEvent
       voteme::on_vote_received(|event| {
           let vote = event.vote();
           println!("Vote received!");
           println!("  Service: {}", vote.service_name());
           println!("  Username: {}", vote.username());
           println!("  Address: {}", vote.address());
           println!("  Timestamp: {}", vote.timestamp());
       }).await;

       Ok(())
   }

3. Event struct yang tersedia:
   - VoteReceivedEvent: Main event struct
     - vote: Vote instance
     - received_at: SystemTime of when the event was received
   
   - Vote: Vote data
     - service_name: String - nama service yang melakukan vote
     - username: String - username yang melakukan vote
     - address: String - IP address dari voter
     - timestamp: u64 - unix timestamp ketika vote dilakukan

4. Methods yang tersedia:
   - event.vote() -> &Vote
   - event.received_at() -> SystemTime
   - vote.service_name() -> &str
   - vote.username() -> &str
   - vote.address() -> &str
   - vote.timestamp() -> u64
*/
