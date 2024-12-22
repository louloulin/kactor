use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::SendError;

#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    pub max_queue_size: usize,
    pub high_watermark: f64,    // 例如 0.8 表示 80%
    pub low_watermark: f64,     // 例如 0.6 表示 60%
    pub pressure_window: Duration,
    pub sampling_interval: Duration,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            high_watermark: 0.8,
            low_watermark: 0.6,
            pressure_window: Duration::from_secs(1),
            sampling_interval: Duration::from_millis(100),
        }
    }
}

pub struct BackpressureController {
    config: BackpressureConfig,
    current_size: AtomicUsize,
    pressure_start: RwLock<Option<Instant>>,
    is_pressured: AtomicBool,
}

impl BackpressureController {
    pub fn new(config: BackpressureConfig) -> Self {
        Self {
            config,
            current_size: AtomicUsize::new(0),
            pressure_start: RwLock::new(None),
            is_pressured: AtomicBool::new(false),
        }
    }

    pub async fn try_acquire(&self) -> Result<(), SendError> {
        let current = self.current_size.load(Ordering::Relaxed);
        
        if current >= self.config.max_queue_size {
            return Err(SendError::MailboxFull);
        }

        if current as f64 >= self.config.max_queue_size as f64 * self.config.high_watermark {
            let mut pressure_start = self.pressure_start.write().await;
            if pressure_start.is_none() {
                *pressure_start = Some(Instant::now());
            }

            if pressure_start.unwrap().elapsed() > self.config.pressure_window {
                self.is_pressured.store(true, Ordering::Release);
                return Err(SendError::BackPressure);
            }
        } else if current as f64 <= self.config.max_queue_size as f64 * self.config.low_watermark {
            *self.pressure_start.write().await = None;
            self.is_pressured.store(false, Ordering::Release);
        }

        self.current_size.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn release(&self) {
        self.current_size.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn is_under_pressure(&self) -> bool {
        self.is_pressured.load(Ordering::Acquire)
    }

    pub fn current_load(&self) -> f64 {
        let current = self.current_size.load(Ordering::Relaxed);
        current as f64 / self.config.max_queue_size as f64
    }
} 