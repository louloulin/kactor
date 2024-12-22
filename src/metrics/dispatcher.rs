use std::sync::Arc;
use std::time::Duration;
use metrics::{Counter, Gauge, Histogram};

#[derive(Debug, Clone)]
pub struct DispatcherMetrics {
    active_tasks: Counter,
    queued_tasks: Gauge,
    completed_tasks: Counter,
    failed_tasks: Counter,
    processing_time: Histogram,
    backpressure_ratio: Gauge,
    worker_utilization: Vec<Gauge>,
}

impl DispatcherMetrics {
    pub fn new(worker_count: usize) -> Self {
        Self {
            active_tasks: Counter::new("dispatcher_active_tasks"),
            queued_tasks: Gauge::new("dispatcher_queued_tasks"),
            completed_tasks: Counter::new("dispatcher_completed_tasks"),
            failed_tasks: Counter::new("dispatcher_failed_tasks"),
            processing_time: Histogram::new("dispatcher_processing_time"),
            backpressure_ratio: Gauge::new("dispatcher_backpressure_ratio"),
            worker_utilization: (0..worker_count)
                .map(|id| Gauge::new(&format!("dispatcher_worker_{}_utilization", id)))
                .collect(),
        }
    }

    pub fn record_task_queued(&self) {
        self.queued_tasks.increment(1.0);
    }

    pub fn record_task_started(&self) {
        self.active_tasks.increment(1);
        self.queued_tasks.decrement(1.0);
    }

    pub fn record_task_completed(&self, duration: Duration, success: bool) {
        self.active_tasks.decrement(1);
        if success {
            self.completed_tasks.increment(1);
        } else {
            self.failed_tasks.increment(1);
        }
        self.processing_time.record(duration.as_secs_f64());
    }

    pub fn update_backpressure(&self, ratio: f64) {
        self.backpressure_ratio.set(ratio);
    }

    pub fn update_worker_utilization(&self, worker_id: usize, utilization: f64) {
        if let Some(gauge) = self.worker_utilization.get(worker_id) {
            gauge.set(utilization);
        }
    }
} 