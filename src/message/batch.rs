use std::time::Duration;
use tokio::time::Instant;
use super::Message;

/// A batch of messages with timing information
pub struct MessageBatch {
    /// The messages in this batch
    messages: Vec<Message>,
    /// When this batch was created
    created_at: Instant,
    /// Maximum size of the batch
    max_size: usize,
    /// Maximum age of the batch
    max_age: Duration,
}

impl MessageBatch {
    /// Creates a new message batch
    pub fn new(max_size: usize, max_age: Duration) -> Self {
        Self {
            messages: Vec::with_capacity(max_size),
            created_at: Instant::now(),
            max_size,
            max_age,
        }
    }

    /// Adds a message to the batch
    pub fn add(&mut self, msg: Message) -> bool {
        if self.is_full() {
            return false;
        }
        self.messages.push(msg);
        true
    }

    /// Returns whether the batch is ready to be processed
    pub fn is_ready(&self) -> bool {
        self.is_full() || self.is_expired()
    }

    /// Returns whether the batch is full
    pub fn is_full(&self) -> bool {
        self.messages.len() >= self.max_size
    }

    /// Returns whether the batch has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.max_age
    }

    /// Takes the messages from this batch
    pub fn take(&mut self) -> Vec<Message> {
        std::mem::take(&mut self.messages)
    }
} 