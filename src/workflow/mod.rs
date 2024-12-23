pub mod actors;

// 配置类型
#[derive(Debug, Clone)]
pub struct SourceConfig {
    pub connection_string: String,
    pub batch_size: usize,
}

#[derive(Debug, Clone)]
pub struct SinkConfig {
    pub destination: String,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone)]
pub enum RetryPolicy {
    NoRetry,
    Fixed { attempts: u32, delay_ms: u64 },
    Exponential { max_attempts: u32, initial_delay_ms: u64 },
}

// 重导出主要类型
pub use self::actors::{SourceActor, TransformActor, SinkActor}; 