use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct LogSuppressionManager {
    suppression_window: Duration,
    logged_keys: RwLock<HashMap<String, Instant>>,
}

impl LogSuppressionManager {
    pub fn new(suppression_window: Duration) -> Self {
        Self {
            suppression_window,
            logged_keys: RwLock::new(HashMap::new()),
        }
    }

    pub fn should_log_warning(&self, warning_key: &str) -> bool {
        let now = Instant::now();
        let mut keys = self.logged_keys.write().unwrap();

        if let Some(last_logged) = keys.get(warning_key) {
            if now.duration_since(*last_logged) < self.suppression_window {
                return false;
            }
        }

        keys.insert(warning_key.to_string(), now);
        true
    }
}
