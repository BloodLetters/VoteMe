pub mod detector;
pub mod proxy;
pub mod v1;
pub mod v2;
pub mod version;

pub use detector::{detect_protocol_version, PROTOCOL_2_MAGIC};
pub use proxy::{process_proxy_headers, ProxyHeaderResult};
pub use v1::{format_v1_payload, parse_v1_decrypted_payload, parse_v1_packet};
pub use v2::{format_v2_packet, parse_v2_packet, V2InnerPayload, V2OuterMessage};
pub use version::VoteProtocolVersion;
