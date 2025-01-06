pub mod strategy;
pub use strategy::*;
#[derive(Default)]
pub struct ChildStats {
    pub failure_count: i32,
    pub last_failure: Option<std::time::Instant>,
    pub restart_count: i32,
}

impl ChildStats {
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(std::time::Instant::now());
    }

    pub fn record_restart(&mut self) {
        self.restart_count += 1;
    }

    pub fn reset(&mut self) {
        self.failure_count = 0;
        self.last_failure = None;
        self.restart_count = 0;
    }
}

pub struct DefaultStrategy;

impl DefaultStrategy {
    // Implement any necessary methods for the default strategy
} 