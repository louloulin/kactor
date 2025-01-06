use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use crate::context::Context;
use crate::message::Message;
use crate::errors::SendError;
use crate::actor::Actor;    
use crate::middleware::Middleware;
// 日志中间件
pub struct LoggingMiddleware {
    log_level: log::Level,
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle<A: Actor>(
        &self,
        ctx: &mut Context,
        msg: Message,
        next: Arc<dyn Fn(Message) -> Result<(), SendError> + Send + Sync>,
    ) -> Result<(), SendError> {
        log::log!(self.log_level, "Processing message: {:?}", msg);
        let result = next(msg).await;
        log::log!(self.log_level, "Message processed with result: {:?}", result);
        result
    }
}

// 性能监控中间件
pub struct MetricsMiddleware {
    metrics: Arc<Metrics>,
}

#[async_trait]
impl Middleware for MetricsMiddleware {
    async fn handle<A: Actor>(
        &self,
        ctx: &mut Context,
        msg: Message,
        next: Arc<dyn Fn(Message) -> Result<(), SendError> + Send + Sync>,
    ) -> Result<(), SendError> {
        let start = Instant::now();
        let result = next(msg).await;
        let duration = start.elapsed();
        
        self.metrics.record_message_processing(duration);
        if result.is_err() {
            self.metrics.record_failure();
        }
        
        result
    }
}

// 重试中间件
pub struct RetryMiddleware {
    max_retries: usize,
    backoff: ExponentialBackoff,
}

#[async_trait]
impl Middleware for RetryMiddleware {
    async fn handle<A: Actor>(
        &self,
        ctx: &mut Context,
        msg: Message,
        next: Arc<dyn Fn(Message) -> Result<(), SendError> + Send + Sync>,
    ) -> Result<(), SendError> {
        let mut retries = 0;
        loop {
            match next(msg.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) if retries < self.max_retries => {
                    retries += 1;
                    let delay = self.backoff.get_delay(retries);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

// 断路器中间件
pub struct CircuitBreakerMiddleware {
    state: Arc<RwLock<CircuitBreakerState>>,
    config: CircuitBreakerConfig,
}

#[async_trait]
impl Middleware for CircuitBreakerMiddleware {
    async fn handle<A: Actor>(
        &self,
        ctx: &mut Context,
        msg: Message,
        next: Arc<dyn Fn(Message) -> Result<(), SendError> + Send + Sync>,
    ) -> Result<(), SendError> {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Open => {
                Err(SendError::CircuitBreakerOpen)
            }
            CircuitBreakerState::HalfOpen => {
                let result = next(msg).await;
                if result.is_ok() {
                    *self.state.write().await = CircuitBreakerState::Closed;
                } else {
                    *self.state.write().await = CircuitBreakerState::Open;
                }
                result
            }
            CircuitBreakerState::Closed => {
                next(msg).await
            }
        }
    }
} 