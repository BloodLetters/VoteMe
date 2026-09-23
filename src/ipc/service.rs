use std::collections::HashSet;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::model::Vote;

pub const EVENT_VOTE_RECEIVED: &str = "vote_received";

#[derive(Debug, Serialize)]
pub struct VoteIpcEvent<'a> {
    pub event: &'static str,
    pub vote: &'a Vote,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteIpcPayload {
    pub event: String,
    pub vote: Vote,
}

#[derive(Debug, Deserialize)]
pub struct IpcIncomingCommand {
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub status: String,
    pub message: String,
}

#[derive(Default)]
pub struct VoteIpcService {
    subscribers: RwLock<HashSet<String>>,
}

impl VoteIpcService {
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(HashSet::new()),
        }
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscribers
            .read()
            .map(|guard| guard.len())
            .unwrap_or(0)
    }

    pub fn is_subscribed(&self, plugin_id: &str) -> bool {
        self.subscribers
            .read()
            .map(|guard| guard.contains(plugin_id))
            .unwrap_or(false)
    }

    pub fn add_subscriber(&self, plugin_id: impl Into<String>) {
        if let Ok(mut guard) = self.subscribers.write() {
            guard.insert(plugin_id.into());
        }
    }

    pub fn remove_subscriber(&self, plugin_id: &str) {
        if let Ok(mut guard) = self.subscribers.write() {
            guard.remove(plugin_id);
        }
    }

    pub fn handle_incoming_message(&self, sender: &str, message_bytes: &[u8]) -> Result<Vec<u8>, String> {
        let incoming: IpcIncomingCommand = serde_json::from_slice(message_bytes)
            .map_err(|e| format!("Invalid IPC JSON request: {e}"))?;

        match incoming.action.to_ascii_lowercase().as_str() {
            "subscribe" => {
                self.add_subscriber(sender);
                let response = IpcResponse {
                    status: "ok".to_string(),
                    message: format!("Plugin '{sender}' subscribed to vote events."),
                };
                serde_json::to_vec(&response).map_err(|e| e.to_string())
            }
            "unsubscribe" => {
                self.remove_subscriber(sender);
                let response = IpcResponse {
                    status: "ok".to_string(),
                    message: format!("Plugin '{sender}' unsubscribed from vote events."),
                };
                serde_json::to_vec(&response).map_err(|e| e.to_string())
            }
            "ping" => {
                let response = IpcResponse {
                    status: "ok".to_string(),
                    message: "pong".to_string(),
                };
                serde_json::to_vec(&response).map_err(|e| e.to_string())
            }
            other => Err(format!(
                "Unknown IPC action '{other}'. Supported actions: subscribe, unsubscribe, ping"
            )),
        }
    }

    pub fn dispatch_vote_event(&self, vote: &Vote) {
        let targets: Vec<String> = match self.subscribers.read() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => return,
        };

        if targets.is_empty() {
            return;
        }

        let event_payload = VoteIpcEvent {
            event: EVENT_VOTE_RECEIVED,
            vote,
        };

        let payload_bytes = match serde_json::to_vec(&event_payload) {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!("[VoteMe] Failed to serialize IPC vote payload: {err}");
                return;
            }
        };

        for recipient in targets {
            match pumpkin_plugin_api::ipc::send_ipc_message(&recipient, &payload_bytes) {
                Ok(Ok(_)) => {}
                Ok(Err(err_msg)) => {
                    eprintln!("[VoteMe] IPC delivery to '{recipient}' returned error: {err_msg}");
                }
                Err(()) => {
                    eprintln!("[VoteMe] Host failed to deliver IPC message to '{recipient}'. Plugin might not be loaded.");
                }
            }
        }
    }
}
