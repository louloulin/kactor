use std::time::Duration;
use tokio::time::Instant;

pub struct BatchMessage<T> {
    pub items: Vec<T>,
    pub created_at: Instant,
    pub timeout: Duration,
}

impl<T> BatchMessage<T> {
    pub fn new(timeout: Duration) -> Self {
        Self {
            items: Vec::new(),
            created_at: Instant::now(),
            timeout,
        }
    }

    pub fn add(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn is_timeout(&self) -> bool {
        self.created_at.elapsed() >= self.timeout
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn take(&mut self) -> Vec<T> {
        std::mem::take(&mut self.items)
    }
} 