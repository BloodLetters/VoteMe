use std::sync::Mutex;
use pumpkin::plugin::Payload;

#[derive(Clone)]
pub struct Vote {
    pub service_name: String,
    pub username: String,
    pub address: String,
    pub timestamp: String,
}

pub struct VoteService {
    listeners: Mutex<Vec<Box<dyn Fn(Vote) + Send + Sync>>>,
}

impl VoteService {
    pub fn new() -> Self {
        Self { listeners: Mutex::new(vec![]) }
    }

    pub fn on_vote<F>(&self, f: F)
    where
        F: Fn(Vote) + Send + Sync + 'static,
    {
        self.listeners.lock().unwrap().push(Box::new(f));
    }

    pub fn emit(&self, vote: Vote) {
        for l in self.listeners.lock().unwrap().iter() {
            l(vote.clone());
        }
    }
}

impl Payload for VoteService {
    fn get_name_static() -> &'static str { "VoteService" }
    fn get_name(&self) -> &'static str { "VoteService" }
    fn as_any(&self) -> &(dyn std::any::Any + 'static) { self }
    fn as_any_mut(&mut self) -> &mut (dyn std::any::Any + 'static) { self }
}