use metrics::{Counter, Gauge, Histogram};
use std::sync::Arc;

pub struct WorkflowMetrics {
    // Source 指标
    pub source_records_read: Counter,
    pub source_bytes_read: Counter,
    pub source_lag: Gauge,
    pub source_errors: Counter,
    
    // Transform 指标
    pub transform_records_processed: Counter,
    pub transform_processing_time: Histogram,
    pub transform_errors: Counter,
    pub transform_backpressure: Gauge,
    
    // Sink 指标
    pub sink_records_written: Counter,
    pub sink_bytes_written: Counter,
    pub sink_latency: Histogram,
    pub sink_errors: Counter,
    
    // 整体指标
    pub end_to_end_latency: Histogram,
    pub workflow_throughput: Gauge,
    pub memory_usage: Gauge,
}

impl WorkflowMetrics {
    pub fn new() -> Self {
        Self {
            source_records_read: Counter::new("source_records_read_total"),
            source_bytes_read: Counter::new("source_bytes_read_total"),
            source_lag: Gauge::new("source_lag_seconds"),
            source_errors: Counter::new("source_errors_total"),
            transform_records_processed: Counter::new("transform_records_processed_total"),
            transform_processing_time: Histogram::new("transform_processing_time_seconds"),
            transform_errors: Counter::new("transform_errors_total"),
            transform_backpressure: Gauge::new("transform_backpressure"),
            sink_records_written: Counter::new("sink_records_written_total"),
            sink_bytes_written: Counter::new("sink_bytes_written_total"),
            sink_latency: Histogram::new("sink_latency_seconds"),
            sink_errors: Counter::new("sink_errors_total"),
            end_to_end_latency: Histogram::new("end_to_end_latency_seconds"),
            workflow_throughput: Gauge::new("workflow_throughput"),
            memory_usage: Gauge::new("memory_usage"),
        }
    }

    pub fn workflow_started(&self, workflow_id: String) {
        self.workflows_started.increment(1);
        self.active_workflows.increment();
    }

    pub fn workflow_completed(&self, workflow_id: String, status: WorkflowState, duration: Duration) {
        match status {
            WorkflowState::Completed => self.workflows_completed.increment(1),
            WorkflowState::Failed => self.workflows_failed.increment(1),
            _ => {}
        }
        self.active_workflows.decrement();
        self.workflow_duration.record(duration.as_secs_f64());
    }

    pub fn step_started(&self, workflow_id: String, step_name: &str) {
        self.active_steps.increment();
    }

    pub fn step_completed(&self, workflow_id: String, step_name: &str, duration: Duration) {
        self.active_steps.decrement();
        self.step_duration.record(duration.as_secs_f64());
    }
} 