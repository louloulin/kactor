use std::time::{Duration, Instant};
use tokio::time::sleep;

pub(crate) trait DurationExt {
    fn mul_f64(self, rhs: f64) -> Self;
}

impl DurationExt for Duration {
    fn mul_f64(self, rhs: f64) -> Self {
        let secs = self.as_secs_f64() * rhs;
        Duration::from_secs_f64(secs)
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    rate: u32,
    interval: Duration,
    last_check: Instant,
    tokens: u32,
}

impl RateLimiter {
    pub fn new(rate: u32) -> Self {
        Self {
            rate,
            interval: Duration::from_secs(1),
            last_check: Instant::now(),
            tokens: rate,
        }
    }

    pub fn set_rate(&mut self, rate: u32) {
        self.rate = rate;
    }

    pub async fn acquire(&mut self, count: u32) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_check);
        
        // 补充令牌
        if elapsed >= self.interval {
            self.tokens = self.rate;
            self.last_check = now;
        } else {
            let new_tokens = (elapsed.as_secs_f64() * self.rate as f64) as u32;
            self.tokens = (self.tokens + new_tokens).min(self.rate);
        }

        // 如果令牌不足，等待
        if self.tokens < count {
            let wait_time = self.interval.mul_f64(
                (count - self.tokens) as f64 / self.rate as f64
            );
            sleep(wait_time).await;
            self.tokens = self.rate;
        }

        self.tokens -= count;
    }

    pub fn get_rate(&self) -> u32 {
        self.rate
    }

    pub fn get_available_tokens(&self) -> u32 {
        self.tokens
    }

    pub fn reset(&mut self) {
        self.tokens = self.rate;
        self.last_check = Instant::now();
    }
} 