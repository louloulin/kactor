use std::time::Duration;
use tokio::time::sleep;
use async_trait::async_trait;
use super::Actor;
use crate::context::Context;
use crate::message::Message;

pub struct ThrottleDecorator<A: Actor> {
    inner: A,
    rate_limit: Duration,
    last_message: std::time::Instant,
}

#[async_trait]
impl<A: Actor> Actor for ThrottleDecorator<A> {
    async fn receive(&mut self, ctx: &mut Context, msg: Message) {
        let now = std::time::Instant::now();
        let elapsed = now - self.last_message;
        
        if elapsed < self.rate_limit {
            sleep(self.rate_limit - elapsed).await;
        }
        
        self.inner.receive(ctx, msg).await;
        self.last_message = std::time::Instant::now();
    }
}

pub struct RetryDecorator<A: Actor> {
    inner: A,
    max_retries: u32,
    retry_delay: Duration,
}

#[async_trait]
impl<A: Actor> Actor for RetryDecorator<A> {
    async fn receive(&mut self, ctx: &mut Context, msg: Message) {
        let mut retries = 0;
        
        while retries < self.max_retries {
            match self.inner.receive(ctx, msg.clone()).await {
                Ok(_) => return,
                Err(_) => {
                    retries += 1;
                    if retries < self.max_retries {
                        sleep(self.retry_delay).await;
                    }
                }
            }
        }
    }
} 