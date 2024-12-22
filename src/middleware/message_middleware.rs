use async_trait::async_trait;
use crate::{Context, Message, Middleware, Next, SendError};
use std::time::Instant;

pub struct LoggingMiddleware {
    prefix: String,
}

impl LoggingMiddleware {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(&self, ctx: &mut Context, msg: Message, next: Next<'_>) -> Result<(), SendError> {
        let start = Instant::now();
        let result = next.run(ctx, msg.clone()).await;
        let duration = start.elapsed();

        tracing::info!(
            "{} Message {:?} processed in {:?}",
            self.prefix,
            msg.payload.type_id(),
            duration
        );

        result
    }
}

pub struct MetricsMiddleware {
    metrics: Arc<ProtoMetrics>,
}

#[async_trait]
impl Middleware for MetricsMiddleware {
    async fn handle(&self, ctx: &mut Context, msg: Message, next: Next<'_>) -> Result<(), SendError> {
        let start = Instant::now();
        let result = next.run(ctx, msg).await;
        let duration = start.elapsed();

        self.metrics.record_message_duration(
            ctx.actor_type(),
            duration.as_millis() as u64
        ).await;

        result
    }
} 