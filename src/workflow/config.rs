use std::time::Duration;

#[derive(Clone)]
pub struct WorkflowConfig {
    // 节点配置
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub execution_timeout: Duration,
    
    // 并发配置
    pub max_concurrent_steps: usize,
    pub step_buffer_size: usize,
    
    // 监控配置
    pub enable_metrics: bool,
    pub metrics_interval: Duration,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
            execution_timeout: Duration::from_secs(3600),
            max_concurrent_steps: 10,
            step_buffer_size: 1000,
            enable_metrics: true,
            metrics_interval: Duration::from_secs(60),
        }
    }
} 