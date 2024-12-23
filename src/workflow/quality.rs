pub struct DataQualityMonitor {
    rules: Vec<Box<dyn DataQualityRule>>,
    metrics: Arc<QualityMetrics>,
    alert_manager: AlertManager,
}

#[async_trait]
pub trait DataQualityRule: Send + Sync {
    async fn check(&self, data: &[Message]) -> DataQualityResult;
    fn severity(&self) -> Severity;
}

impl DataQualityMonitor {
    pub async fn check_batch(&self, batch: &[Message]) -> Vec<DataQualityResult> {
        let mut results = Vec::new();
        
        for rule in &self.rules {
            let result = rule.check(batch).await;
            if !result.passed {
                self.metrics.record_violation(&result);
                if result.severity >= Severity::High {
                    self.alert_manager.send_alert(&result).await;
                }
            }
            results.push(result);
        }
        
        results
    }
} 