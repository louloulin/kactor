use async_trait::async_trait;

/// A trait for mailbox dispatchers
#[async_trait]
pub trait MailboxDispatcher: Send + Sync {
    async fn dispatch(&self);
}

/// A default dispatcher implementation
pub struct DefaultDispatcher {
    capacity: usize,
}

impl DefaultDispatcher {
    pub fn new(capacity: usize) -> Self {
        Self { capacity }
    }
}

#[async_trait]
impl MailboxDispatcher for DefaultDispatcher {
    async fn dispatch(&self) {
        // Implement dispatch logic here
    }
} 