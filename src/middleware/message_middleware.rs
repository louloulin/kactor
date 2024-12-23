use async_trait::async_trait;
use std::time::Instant;
use std::sync::Arc;
use crate::context::Context;
use crate::message::Message;
use crate::errors::SendError;
use super::{Middleware, Next};

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
        let result = next(ctx, msg.clone()).await;
        let duration = start.elapsed();

        tracing::info!(
            "{} Message processed in {:?}",
            self.prefix,
            duration
        );

        result
    }
} 