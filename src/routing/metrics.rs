use std::sync::Arc;
use metrics::{Counter, Gauge, Histogram};
use dashmap::DashMap;

#[derive(Debug)]
pub struct RouterMetrics {
    routee_count: Gauge,
    message_count: Counter,
    routing_time: Histogram,
    errors: Counter,
    routee_mailbox_sizes: Arc<DashMap<String, Gauge>>,
}

impl RouterMetrics {
    pub fn new(router_id: &str) -> Self {
        Self {
            routee_count: Gauge::new(&format!("router_{}_routee_count", router_id)),
            message_count: Counter::new(&format!("router_{}_message_count", router_id)),
            routing_time: Histogram::new(&format!("router_{}_routing_time", router_id)),
            errors: Counter::new(&format!("router_{}_errors", router_id)),
            routee_mailbox_sizes: Arc::new(DashMap::new()),
        }
    }

    pub fn record_message(&self) {
        self.message_count.increment(1);
    }

    pub fn record_routing_time(&self, duration: std::time::Duration) {
        self.routing_time.record(duration.as_secs_f64());
    }

    pub fn record_error(&self) {
        self.errors.increment(1);
    }

    pub fn update_routee_count(&self, count: usize) {
        self.routee_count.set(count as f64);
    }

    pub fn update_routee_mailbox_size(&self, routee_id: &str, size: usize) {
        self.routee_mailbox_sizes
            .entry(routee_id.to_string())
            .or_insert_with(|| Gauge::new(&format!("routee_{}_mailbox_size", routee_id)))
            .set(size as f64);
    }
} 