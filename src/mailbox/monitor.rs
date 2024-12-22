use super::*;
use std::time::Duration;

pub struct MailboxMonitor {
    metrics: Arc<MailboxMetrics>,
    thresholds: MonitorThresholds,
    alerts: Vec<Box<dyn AlertHandler>>,
}

pub struct MonitorThresholds {
    queue_size_threshold: usize,
    processing_time_threshold: Duration,
    error_rate_threshold: f64,
}

#[async_trait::async_trait]
pub trait AlertHandler: Send + Sync {
    async fn handle_alert(&self, alert: MailboxAlert);
}

#[derive(Debug)]
pub enum MailboxAlert {
    QueueSizeExceeded {
        current: usize,
        threshold: usize,
    },
    ProcessingTimeExceeded {
        current: Duration,
        threshold: Duration,
    },
    ErrorRateExceeded {
        current: f64,
        threshold: f64,
    },
}

impl MailboxMonitor {
    pub fn new(metrics: Arc<MailboxMetrics>, thresholds: MonitorThresholds) -> Self {
        Self {
            metrics,
            thresholds,
            alerts: Vec::new(),
        }
    }

    pub fn add_alert_handler<H: AlertHandler + 'static>(&mut self, handler: H) {
        self.alerts.push(Box::new(handler));
    }

    pub async fn check(&self) {
        let stats = self.metrics.get_stats();
        
        // 检查队列大小
        if stats.messages_queued as usize > self.thresholds.queue_size_threshold {
            self.emit_alert(MailboxAlert::QueueSizeExceeded {
                current: stats.messages_queued as usize,
                threshold: self.thresholds.queue_size_threshold,
            }).await;
        }
        
        // 检查处理时间
        if stats.avg_processing_time > self.thresholds.processing_time_threshold {
            self.emit_alert(MailboxAlert::ProcessingTimeExceeded {
                current: stats.avg_processing_time,
                threshold: self.thresholds.processing_time_threshold,
            }).await;
        }
        
        // 检查错误率
        let error_rate = stats.errors as f64 / stats.messages_processed as f64;
        if error_rate > self.thresholds.error_rate_threshold {
            self.emit_alert(MailboxAlert::ErrorRateExceeded {
                current: error_rate,
                threshold: self.thresholds.error_rate_threshold,
            }).await;
        }
    }

    async fn emit_alert(&self, alert: MailboxAlert) {
        for handler in &self.alerts {
            handler.handle_alert(alert.clone()).await;
        }
    }
} 