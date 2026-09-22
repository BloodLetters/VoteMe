use std::sync::Arc;

use crate::config::VotifierConfig;
use crate::dispatcher::VoteDispatcher;
use crate::model::stats::VoteStatistics;
use crate::model::Vote;

#[test]
fn test_vote_dispatcher_processing() {
    let config = Arc::new(VotifierConfig::default());
    let stats = Arc::new(VoteStatistics::new());
    let dispatcher = VoteDispatcher::new(config, Arc::clone(&stats));

    let valid_vote = Vote::new(
        "PlanetMinecraft",
        "Steve_123",
        "127.0.0.1",
        "1695384000",
        "127.0.0.1",
    );

    let result = dispatcher.process_incoming_vote(&valid_vote);
    assert!(result.accepted);
    assert_eq!(stats.valid_votes(), 1);
    assert!(result.broadcast_message.is_some());
    assert!(!result.commands_to_execute.is_empty());

    let invalid_vote = Vote::new(
        "PlanetMinecraft",
        "Invalid Player Name!",
        "127.0.0.1",
        "1695384000",
        "127.0.0.1",
    );

    let rejected = dispatcher.process_incoming_vote(&invalid_vote);
    assert!(!rejected.accepted);
    assert_eq!(stats.failed_votes(), 1);
    assert!(rejected.broadcast_message.is_none());
    assert!(rejected.commands_to_execute.is_empty());
}
