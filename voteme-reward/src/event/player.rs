use pumpkin_api_macros::with_runtime;
use pumpkin::{
    plugin::{player::player_join::PlayerJoinEvent, Context, EventHandler, EventPriority},
    server::Server,
};
use pumpkin_util::text::{color::NamedColor, TextComponent};

struct VoteMeJoinHandler; 

#[with_runtime(global)]
impl EventHandler<PlayerJoinEvent> for VoteMeJoinHandler {
    fn handle_blocking(&self, _server: &Arc<Server>, event: &mut PlayerJoinEvent) -> BoxFuture<'_, ()> {
        Box::pin(async move {

        // Total Vote Amount Reminder
        event.join_message =
            TextComponent::text(format!("[VoteMe] You have {} total votes!", 0))
                .color_named(NamedColor::Green);
        })

    }
}