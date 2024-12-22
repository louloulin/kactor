use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct RateLimiter {
    rate: AtomicUsize,
    interval: Duration,
    last_reset: parking_lot::Mutex<Instant>,
    current_count: AtomicUsize,
}

impl RateLimiter {
    pub fn new(rate: usize, interval: Duration) -> Self {
        Self {
            rate: AtomicUsize::new(rate),
            interval,
            last_reset: parking_lot::Mutex::new(Instant::now()),
            current_count: AtomicUsize::new(0),
        }
    }

    pub async fn acquire(&self) -> bool {
        let now = Instant::now();
        let mut last_reset = self.last_reset.lock();
        
        if now.duration_since(*last_reset) >= self.interval {
            *last_reset = now;
            self.current_count.store(0, Ordering::SeqCst);
        }

        let current = self.current_count.fetch_add(1, Ordering::SeqCst);
        if current >= self.rate.load(Ordering::SeqCst) {
            sleep(self.interval).await;
            false
        } else {
            true
        }
    }

    pub fn update_rate(&self, new_rate: usize) {
        self.rate.store(new_rate, Ordering::SeqCst);
    }
}

pub struct FlowController {
    message_rate_limiter: RateLimiter,
    byte_rate_limiter: RateLimiter,
}

impl FlowController {
    pub fn new(messages_per_sec: usize, bytes_per_sec: usize) -> Self {
        Self {
            message_rate_limiter: RateLimiter::new(
                messages_per_sec,
                Duration::from_secs(1),
            ),
            byte_rate_limiter: RateLimiter::new(
                bytes_per_sec,
                Duration::from_secs(1),
            ),
        }
    }

    pub async fn can_send(&self, message_size: usize) -> bool {
        self.message_rate_limiter.acquire().await && 
        self.byte_rate_limiter.acquire().await
    }
} 