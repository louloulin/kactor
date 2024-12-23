pub struct BackpressureController {
    max_buffer_size: usize,
    current_buffer_size: AtomicUsize,
    high_watermark: f64,
    low_watermark: f64,
    throttle_sender: mpsc::Sender<ThrottleCommand>,
}

#[derive(Debug)]
pub enum ThrottleCommand {
    Pause,
    Resume,
    UpdateRate(u32),
}

impl BackpressureController {
    pub fn new(max_buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            max_buffer_size,
            current_buffer_size: AtomicUsize::new(0),
            high_watermark: 0.8,
            low_watermark: 0.2,
            throttle_sender: tx,
        }
    }

    pub fn add_records(&self, count: usize) -> bool {
        let current = self.current_buffer_size.load(Ordering::Relaxed);
        let new_size = current + count;
        
        if new_size > (self.max_buffer_size as f64 * self.high_watermark) as usize {
            let _ = self.throttle_sender.try_send(ThrottleCommand::Pause);
            false
        } else {
            self.current_buffer_size.store(new_size, Ordering::Relaxed);
            true
        }
    }

    pub fn remove_records(&self, count: usize) {
        let current = self.current_buffer_size.load(Ordering::Relaxed);
        let new_size = current.saturating_sub(count);
        self.current_buffer_size.store(new_size, Ordering::Relaxed);
        
        if new_size < (self.max_buffer_size as f64 * self.low_watermark) as usize {
            let _ = self.throttle_sender.try_send(ThrottleCommand::Resume);
        }
    }
} 