use std::time::Duration;
use serde::{Serialize, Deserialize};
use super::{Event, Snapshot};

pub struct Recovery {
    pub snapshot: Option<Snapshot>,
    pub events: Vec<Event>,
}

#[derive(Clone)]
pub struct RecoveryStrategy {
    pub snapshot_interval: Option<Duration>,
    pub recovery_timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        Self {
            snapshot_interval: Some(Duration::from_secs(3600)), // 1小时
            recovery_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        }
    }
}

impl RecoveryStrategy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_snapshot_interval(mut self, interval: Option<Duration>) -> Self {
        self.snapshot_interval = interval;
        self
    }

    pub fn with_recovery_timeout(mut self, timeout: Duration) -> Self {
        self.recovery_timeout = timeout;
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }
} 