use std::time::{Duration, Instant};
use crate::message::Message;

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

    fn handle_tumbling_window(&mut self, msg: Message, timestamp: Instant, size: Duration) {
        let window_start = timestamp - (timestamp.duration_since(Instant::now()) % size);
        let window_end = window_start + size;

        if let Some(window) = self.current_windows
            .iter_mut()
            .find(|w| w.start_time == window_start) {
                window.data.push(msg);
        } else {
            let window = Window {
                window_type: WindowType::Tumbling { size },
                start_time: window_start,
                end_time: window_end,
                data: vec![msg],
            };
            self.current_windows.push(window);
        }
    }

    fn handle_sliding_window(&mut self, msg: Message, timestamp: Instant, size: Duration, slide: Duration) {
        let current_windows: Vec<_> = (0..)
            .map(|i| timestamp - (timestamp.duration_since(Instant::now()) % slide) - i * slide)
            .take_while(|start| timestamp.duration_since(*start) < size)
            .collect();

        for window_start in current_windows {
            let window_end = window_start + size;
            if let Some(window) = self.current_windows
                .iter_mut()
                .find(|w| w.start_time == window_start) {
                    window.data.push(msg.clone());
            } else {
                let window = Window {
                    window_type: WindowType::Sliding { size, slide },
                    start_time: window_start,
                    end_time: window_end,
                    data: vec![msg.clone()],
                };
                self.current_windows.push(window);
            }
        }
    }

    fn handle_session_window(&mut self, msg: Message, timestamp: Instant, timeout: Duration) {
        if let Some(window) = self.current_windows
            .iter_mut()
            .find(|w| timestamp.duration_since(w.end_time) < timeout) {
                window.data.push(msg);
                window.end_time = timestamp + timeout;
        } else {
            let window = Window {
                window_type: WindowType::Session { timeout },
                start_time: timestamp,
                end_time: timestamp + timeout,
                data: vec![msg],
            };
            self.current_windows.push(window);
        }
    }

    fn handle_count_window(&mut self, msg: Message, size: usize) {
        if let Some(window) = self.current_windows.last_mut() {
            if window.data.len() < size {
                window.data.push(msg);
                return;
            }
        }
        
        let window = Window {
            window_type: WindowType::Count { size },
            start_time: Instant::now(),
            end_time: Instant::now(),
            data: vec![msg],
        };
        self.current_windows.push(window);
    }
} 