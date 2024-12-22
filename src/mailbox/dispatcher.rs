use super::*;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[async_trait::async_trait]
pub trait MailboxDispatcher: Send + Sync {
    async fn schedule(&self, mailbox: Arc<dyn Mailbox>, ctx: Arc<Context>) -> JoinHandle<()>;
    fn throughput(&self) -> usize;
}

pub struct DefaultDispatcher {
    throughput: usize,
}

impl DefaultDispatcher {
    pub fn new(throughput: usize) -> Self {
        Self { throughput }
    }
}

#[async_trait::async_trait]
impl MailboxDispatcher for DefaultDispatcher {
    async fn schedule(&self, mailbox: Arc<dyn Mailbox>, ctx: Arc<Context>) -> JoinHandle<()> {
        tokio::spawn(async move {
            while mailbox.status() == MailboxStatus::Open {
                // 处理消息批次
                for _ in 0..self.throughput {
                    match mailbox.receive().await {
                        Ok(Some(msg)) => {
                            if let Err(e) = ctx.handle(msg).await {
                                log::error!("Failed to handle message: {:?}", e);
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            log::error!("Error receiving message: {:?}", e);
                            break;
                        }
                    }
                }
                
                // 让出执行权
                tokio::task::yield_now().await;
            }
        })
    }

    fn throughput(&self) -> usize {
        self.throughput
    }
}

pub struct ThrottledDispatcher {
    throughput: usize,
    interval: Duration,
}

#[async_trait::async_trait]
impl MailboxDispatcher for ThrottledDispatcher {
    async fn schedule(&self, mailbox: Arc<dyn Mailbox>, ctx: Arc<Context>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.interval);
            
            while mailbox.status() == MailboxStatus::Open {
                interval.tick().await;
                
                // 处理有限数量的消息
                for _ in 0..self.throughput {
                    match mailbox.receive().await {
                        Ok(Some(msg)) => {
                            if let Err(e) = ctx.handle(msg).await {
                                log::error!("Failed to handle message: {:?}", e);
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            log::error!("Error receiving message: {:?}", e);
                            break;
                        }
                    }
                }
            }
        })
    }

    fn throughput(&self) -> usize {
        self.throughput
    }
}

pub struct BatchDispatcher {
    throughput: usize,
    batch_size: usize,
    batch_interval: Duration,
}

impl BatchDispatcher {
    pub fn new(throughput: usize, batch_size: usize, batch_interval: Duration) -> Self {
        Self {
            throughput,
            batch_size,
            batch_interval,
        }
    }
}

#[async_trait::async_trait]
impl MailboxDispatcher for BatchDispatcher {
    async fn schedule(&self, mailbox: Arc<dyn Mailbox>, ctx: Arc<Context>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.batch_interval);
            let mut batch = Vec::with_capacity(self.batch_size);
            
            while mailbox.status() == MailboxStatus::Open {
                interval.tick().await;
                
                // 收集一批消息
                for _ in 0..self.batch_size {
                    match mailbox.receive().await {
                        Ok(Some(msg)) => batch.push(msg),
                        _ => break,
                    }
                }
                
                // 批量处理消息
                if !batch.is_empty() {
                    if let Err(e) = ctx.handle_batch(&batch).await {
                        log::error!("Failed to handle batch: {:?}", e);
                    }
                    batch.clear();
                }
                
                tokio::task::yield_now().await;
            }
        })
    }

    fn throughput(&self) -> usize {
        self.throughput
    }
}

pub struct PriorityDispatcher {
    throughput: usize,
    high_priority_ratio: f32,
}

impl PriorityDispatcher {
    pub fn new(throughput: usize, high_priority_ratio: f32) -> Self {
        Self {
            throughput,
            high_priority_ratio: high_priority_ratio.clamp(0.0, 1.0),
        }
    }
}

#[async_trait::async_trait]
impl MailboxDispatcher for PriorityDispatcher {
    async fn schedule(&self, mailbox: Arc<dyn Mailbox>, ctx: Arc<Context>) -> JoinHandle<()> {
        tokio::spawn(async move {
            while mailbox.status() == MailboxStatus::Open {
                let high_priority_count = (self.throughput as f32 * self.high_priority_ratio) as usize;
                
                // 处理高优先级消息
                for _ in 0..high_priority_count {
                    if let Ok(Some(msg)) = mailbox.receive_priority(MessagePriority::High).await {
                        if let Err(e) = ctx.handle(msg).await {
                            log::error!("Failed to handle high priority message: {:?}", e);
                        }
                    }
                }
                
                // 处理普通优先级消息
                for _ in 0..(self.throughput - high_priority_count) {
                    if let Ok(Some(msg)) = mailbox.receive().await {
                        if let Err(e) = ctx.handle(msg).await {
                            log::error!("Failed to handle message: {:?}", e);
                        }
                    }
                }
                
                tokio::task::yield_now().await;
            }
        })
    }

    fn throughput(&self) -> usize {
        self.throughput
    }
} 