use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub struct MailboxMetrics {
    // 消息计数
    messages_processed: AtomicU64,
    messages_queued: AtomicU64,
    messages_dropped: AtomicU64,
    
    // 延迟统计
    processing_time: AtomicU64,  // 纳秒
    queuing_time: AtomicU64,     // 纳秒
    
    // 错误计数
    errors: AtomicU64,
    
    // 状态变更
    status_changes: AtomicU64,
}

impl MailboxMetrics {
    pub fn new() -> Self {
        Self {
            messages_processed: AtomicU64::new(0),
            messages_queued: AtomicU64::new(0),
            messages_dropped: AtomicU64::new(0),
            processing_time: AtomicU64::new(0),
            queuing_time: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            status_changes: AtomicU64::new(0),
        }
    }

    pub fn record_message_processed(&self, processing_time: Duration) {
        self.messages_processed.fetch_add(1, Ordering::Relaxed);
        self.processing_time.fetch_add(processing_time.as_nanos() as u64, Ordering::Relaxed);
    }

    pub fn record_message_queued(&self, queuing_time: Duration) {
        self.messages_queued.fetch_add(1, Ordering::Relaxed);
        self.queuing_time.fetch_add(queuing_time.as_nanos() as u64, Ordering::Relaxed);
    }

    pub fn record_message_dropped(&self) {
        self.messages_dropped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_status_change(&self) {
        self.status_changes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> MailboxStats {
        MailboxStats {
            messages_processed: self.messages_processed.load(Ordering::Relaxed),
            messages_queued: self.messages_queued.load(Ordering::Relaxed),
            messages_dropped: self.messages_dropped.load(Ordering::Relaxed),
            avg_processing_time: Duration::from_nanos(
                self.processing_time.load(Ordering::Relaxed) / 
                self.messages_processed.load(Ordering::Relaxed).max(1)
            ),
            avg_queuing_time: Duration::from_nanos(
                self.queuing_time.load(Ordering::Relaxed) / 
                self.messages_queued.load(Ordering::Relaxed).max(1)
            ),
            errors: self.errors.load(Ordering::Relaxed),
            status_changes: self.status_changes.load(Ordering::Relaxed),
        }
    }
}

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