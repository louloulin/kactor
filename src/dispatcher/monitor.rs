use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use crate::metrics::MetricsCollector;

pub(crate) struct DispatcherMonitor {
    dispatcher: Arc<dyn Dispatcher>,
    metrics_collector: Arc<dyn MetricsCollector>,
    check_interval: Duration,
}

impl DispatcherMonitor {
    pub fn new(
        dispatcher: Arc<dyn Dispatcher>,
        metrics_collector: Arc<dyn MetricsCollector>,
        check_interval: Duration,
    ) -> Self {
        Self {
            dispatcher,
            metrics_collector,
            check_interval,
        }
    }

    pub async fn start(self) {
        let mut interval = interval(self.check_interval);

        loop {
            interval.tick().await;
            
            let metrics = self.dispatcher.metrics();
            
            // 收集指标
            self.metrics_collector.record_gauge(
                "dispatcher_active_threads",
                metrics.active_threads as f64,
            );
            
            self.metrics_collector.record_gauge(
                "dispatcher_queued_tasks",
                metrics.queued_tasks as f64,
            );
            
            self.metrics_collector.record_counter(
                "dispatcher_completed_tasks",
                metrics.completed_tasks as u64,
            );
            
            self.metrics_collector.record_counter(
                "dispatcher_failed_tasks",
                metrics.failed_tasks as u64,
            );
            
            self.metrics_collector.record_histogram(
                "dispatcher_processing_time",
                metrics.avg_processing_time.as_secs_f64(),
            );
        }
    }
} 