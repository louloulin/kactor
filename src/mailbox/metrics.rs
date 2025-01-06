use std::sync::Arc;
use std::time::Duration;

/// Metrics for mailbox performance
#[derive(Debug, Clone)]
pub struct MailboxMetrics {
    // Add fields for metrics tracking
    pub messages_processed: u64,
    pub messages_queued: u64,
    pub messages_dropped: u64,
    pub avg_processing_time: Duration,
    pub avg_queuing_time: Duration,
    pub errors: u64,
    pub status_changes: u64,
}

impl MailboxMetrics {
    pub fn new() -> Self {
        Self {
            messages_processed: 0,
            messages_queued: 0,
            messages_dropped: 0,
            avg_processing_time: Duration::new(0, 0),
            avg_queuing_time: Duration::new(0, 0),
            errors: 0,
            status_changes: 0,
        }
    }

    // Add methods for updating and retrieving metrics
}

/// Statistics for mailbox performance
#[derive(Debug, Clone)]
pub struct MailboxStats {
    pub messages_processed: u64,
    pub messages_queued: u64,
    pub messages_dropped: u64,
    pub avg_processing_time: Duration,
    pub avg_queuing_time: Duration,
    pub errors: u64,
    pub status_changes: u64,
} 