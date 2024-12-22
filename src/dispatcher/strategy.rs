use std::sync::atomic::{AtomicUsize, Ordering};
use rand::Rng;
use dashmap::DashMap;
use tokio::sync::RwLock;
use crate::{Context, Message, SendError};

// 轮询调度器状态
pub(crate) struct RoundRobinState {
    current: AtomicUsize,
    worker_count: usize,
}

impl RoundRobinState {
    pub fn new(worker_count: usize) -> Self {
        Self {
            current: AtomicUsize::new(0),
            worker_count,
        }
    }

    pub fn next_worker(&self) -> usize {
        let current = self.current.fetch_add(1, Ordering::SeqCst);
        current % self.worker_count
    }
}

// 最小负载状态
pub(crate) struct LeastBusyState {
    worker_loads: DashMap<usize, usize>,
}

impl LeastBusyState {
    pub fn new(worker_count: usize) -> Self {
        let worker_loads = DashMap::new();
        for i in 0..worker_count {
            worker_loads.insert(i, 0);
        }
        Self { worker_loads }
    }

    pub fn get_least_busy_worker(&self) -> usize {
        self.worker_loads
            .iter()
            .min_by_key(|entry| *entry.value())
            .map(|entry| *entry.key())
            .unwrap_or(0)
    }

    pub fn update_load(&self, worker_id: usize, delta: i32) {
        if let Some(mut load) = self.worker_loads.get_mut(&worker_id) {
            *load = (*load as i32 + delta).max(0) as usize;
        }
    }
} 