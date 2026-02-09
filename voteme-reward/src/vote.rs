use std::any::Any;
use pumpkin::plugin::Payload;

#[derive(Clone, Debug)]
pub struct Vote {
    pub service_name: String,
    pub username: String,
    pub address: String,
    pub timestamp: String,
}

impl Payload for Vote {
    fn get_name_static() -> &'static str { "Vote" }
    fn get_name(&self) -> &'static str { "Vote" }
    fn as_any(&self) -> &(dyn Any + 'static) { self }
    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) { self }
}