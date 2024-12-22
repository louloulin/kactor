use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct FailureDetector {
    heartbeat_interval: Duration,
    failure_threshold: u32,
    last_heartbeats: HashMap<String, Instant>,
}

impl FailureDetector {
    pub fn new(heartbeat_interval: Duration, failure_threshold: u32) -> Self {
        Self {
            heartbeat_interval,
            failure_threshold,
            last_heartbeats: HashMap::new(),
        }
    }

    pub fn heartbeat(&mut self, member_id: &str) {
        self.last_heartbeats.insert(member_id.to_string(), Instant::now());
    }

    pub fn is_failed(&self, member_id: &str, now: Instant) -> bool {
        if let Some(last_heartbeat) = self.last_heartbeats.get(member_id) {
            let elapsed = now.duration_since(*last_heartbeat);
            elapsed > self.heartbeat_interval * self.failure_threshold
        } else {
            true
        }
    }

    pub fn remove_member(&mut self, member_id: &str) {
        self.last_heartbeats.remove(member_id);
    }
} 