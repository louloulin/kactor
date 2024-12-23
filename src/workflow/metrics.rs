use metrics::{Counter, Gauge, Histogram};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct SourceMetrics {
    pub records_read: Counter,
    pub bytes_read: Counter,
    pub read_latency: Histogram,
    pub current_lag: Gauge,
    pub errors: Counter,
}

#[derive(Clone)]
pub struct TransformMetrics {
    pub records_processed: Counter,
    pub processing_time: Histogram,
    pub memory_usage: Gauge,
    pub errors: Counter,
}

#[derive(Clone)]
pub struct SinkMetrics {
    pub records_written: Counter,
    pub bytes_written: Counter,
    pub write_latency: Histogram,
    pub errors: Counter,
}

impl SourceMetrics {
    pub fn new(prefix: &str) -> Self {
        Self {
            records_read: Counter::new(format!("{}_records_read_total", prefix)),
            bytes_read: Counter::new(format!("{}_bytes_read_total", prefix)),
            read_latency: Histogram::new(format!("{}_read_latency_seconds", prefix)),
            current_lag: Gauge::new(format!("{}_current_lag", prefix)),
            errors: Counter::new(format!("{}_errors_total", prefix)),
        }
    }
}

pub struct WorkflowMetrics {
    // Source 指标
    pub source_metrics: SourceMetrics,
    // Transform 指标
    pub transform_metrics: TransformMetrics,
    // Sink 指标
    pub sink_metrics: SinkMetrics,
    // 整体指标
    pub end_to_end_latency: Histogram,
    pub workflow_throughput: Gauge,
    pub memory_usage: Gauge,
}

impl WorkflowMetrics {
    pub fn new(workflow_id: &str) -> Self {
        Self {
            source_metrics: SourceMetrics::new(&format!("{}_source", workflow_id)),
            transform_metrics: TransformMetrics::new(&format!("{}_transform", workflow_id)),
            sink_metrics: SinkMetrics::new(&format!("{}_sink", workflow_id)),
            end_to_end_latency: Histogram::new(&format!("{}_latency", workflow_id)),
            workflow_throughput: Gauge::new(&format!("{}_throughput", workflow_id)),
            memory_usage: Gauge::new(&format!("{}_memory", workflow_id)),
        }
    }

    pub fn record_source_metrics(&self, records: u64, bytes: u64, latency: Duration) {
        self.source_metrics.records_read.increment(records);
        self.source_metrics.bytes_read.increment(bytes);
        self.source_metrics.current_lag.set(latency.as_secs_f64());
    }

    pub fn record_transform_metrics(&self, records: u64, processing_time: Duration) {
        self.transform_metrics.records_processed.increment(records);
        self.transform_metrics.processing_time.record(processing_time.as_secs_f64());
    }

    pub fn record_sink_metrics(&self, records: u64, bytes: u64, latency: Duration) {
        self.sink_metrics.records_written.increment(records);
        self.sink_metrics.bytes_written.increment(bytes);
        self.sink_metrics.write_latency.record(latency.as_secs_f64());
    }
} 