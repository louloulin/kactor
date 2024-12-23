use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum WindowType {
    Tumbling { size: Duration },
    Sliding { size: Duration, slide: Duration },
    Session { timeout: Duration },
    Count { size: usize },
}

pub struct Window {
    pub window_type: WindowType,
    pub start_time: Instant,
    pub end_time: Instant,
    pub data: Vec<Message>,
}

pub struct WindowManager {
    window_type: WindowType,
    current_windows: Vec<Window>,
    watermark: Instant,
}

impl WindowManager {
    pub fn new(window_type: WindowType) -> Self {
        Self {
            window_type,
            current_windows: Vec::new(),
            watermark: Instant::now(),
        }
    }

    pub fn add_element(&mut self, msg: Message, timestamp: Instant) {
        match &self.window_type {
            WindowType::Tumbling { size } => {
                self.handle_tumbling_window(msg, timestamp, *size);
            }
            WindowType::Sliding { size, slide } => {
                self.handle_sliding_window(msg, timestamp, *size, *slide);
            }
            WindowType::Session { timeout } => {
                self.handle_session_window(msg, timestamp, *timeout);
            }
            WindowType::Count { size } => {
                self.handle_count_window(msg, *size);
            }
        }
    }

    pub fn get_ready_windows(&mut self) -> Vec<Window> {
        self.current_windows.drain_filter(|w| w.end_time <= self.watermark).collect()
    }
} 