pub mod service;

pub use service::{
    EVENT_VOTE_RECEIVED, IpcIncomingCommand, IpcResponse, VoteIpcEvent, VoteIpcPayload,
    VoteIpcService,
};
