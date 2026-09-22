use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ThrottleOptions {
    pub enabled: bool,
    pub max_failures_before_throttle: u32,
    pub throttle_duration: Duration,
    pub max_failures_before_ban: u32,
    pub ban_duration: Duration,
    pub failure_reset_window: Duration,
}

impl Default for ThrottleOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            max_failures_before_throttle: 5,
            throttle_duration: Duration::from_secs(300),
            max_failures_before_ban: 10,
            ban_duration: Duration::from_secs(900),
            failure_reset_window: Duration::from_secs(120),
        }
    }
}

#[derive(Debug, Clone)]
struct ClientTrackState {
    first_failure_time: Instant,
    last_failure_time: Instant,
    failure_count: u32,
    throttled_until: Option<Instant>,
    banned_until: Option<Instant>,
}

impl ClientTrackState {
    fn new(now: Instant) -> Self {
        Self {
            first_failure_time: now,
            last_failure_time: now,
            failure_count: 1,
            throttled_until: None,
            banned_until: None,
        }
    }
}

pub struct VoteThrottleService {
    options: ThrottleOptions,
    client_states: RwLock<HashMap<String, ClientTrackState>>,
}

impl VoteThrottleService {
    pub fn new(options: ThrottleOptions) -> Self {
        Self {
            options,
            client_states: RwLock::new(HashMap::new()),
        }
    }

    pub fn check_client_allowed(&self, client_ip: &str) -> Result<(), u64> {
        if !self.options.enabled {
            return Ok(());
        }

        let now = Instant::now();
        let states = self.client_states.read().unwrap();

        if let Some(state) = states.get(client_ip) {
            if let Some(banned_until) = state.banned_until {
                if now < banned_until {
                    let remaining = banned_until.duration_since(now).as_secs();
                    return Err(remaining.max(1));
                }
            }

            if let Some(throttled_until) = state.throttled_until {
                if now < throttled_until {
                    let remaining = throttled_until.duration_since(now).as_secs();
                    return Err(remaining.max(1));
                }
            }
        }

        Ok(())
    }

    pub fn record_failure(&self, client_ip: &str) {
        if !self.options.enabled {
            return;
        }

        let now = Instant::now();
        let mut states = self.client_states.write().unwrap();

        let state = states
            .entry(client_ip.to_string())
            .and_modify(|state| {
                if now.duration_since(state.last_failure_time) > self.options.failure_reset_window {
                    state.first_failure_time = now;
                    state.failure_count = 1;
                } else {
                    state.failure_count += 1;
                }
                state.last_failure_time = now;
            })
            .or_insert_with(|| ClientTrackState::new(now));

        if state.failure_count >= self.options.max_failures_before_ban {
            state.banned_until = Some(now + self.options.ban_duration);
        } else if state.failure_count >= self.options.max_failures_before_throttle {
            state.throttled_until = Some(now + self.options.throttle_duration);
        }
    }

    pub fn record_success(&self, client_ip: &str) {
        if !self.options.enabled {
            return;
        }

        let mut states = self.client_states.write().unwrap();
        states.remove(client_ip);
    }

    pub fn cleanup_expired_records(&self) {
        let now = Instant::now();
        let mut states = self.client_states.write().unwrap();

        states.retain(|_, state| {
            let is_banned = state.banned_until.map_or(false, |until| now < until);
            let is_throttled = state.throttled_until.map_or(false, |until| now < until);
            let is_recent = now.duration_since(state.last_failure_time) <= self.options.failure_reset_window;

            is_banned || is_throttled || is_recent
        });
    }
}
