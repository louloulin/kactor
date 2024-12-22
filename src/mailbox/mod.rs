mod queue;
mod priority;
mod bounded;
mod unbounded;

pub use queue::*;
pub use priority::*;
pub use bounded::*;
pub use unbounded::*;

use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use crate::{Actor, Context, Message, SystemMessage, SendError};
use crate::mailbox::dispatcher::{MailboxDispatcher, DefaultDispatcher};
use crate::mailbox::metrics::{MailboxMetrics, MailboxStats};

/// Mailbox 配置
#[derive(Debug, Clone)]
pub struct MailboxConfig {
    pub capacity: usize,
    pub throughput: usize,
    pub dispatcher: Arc<dyn MailboxDispatcher>,
    pub metrics_enabled: bool,
}

impl Default for MailboxConfig {
    fn default() -> Self {
        Self {
            capacity: 1000,
            throughput: 100,
            dispatcher: Arc::new(DefaultDispatcher::new(100)),
            metrics_enabled: true,
        }
    }
}

/// Mailbox 状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MailboxStatus {
    Open,
    Closed,
    Suspended,
}

/// Mailbox 特征
#[async_trait::async_trait]
pub trait Mailbox: Send + Sync {
    /// 发送用户消息
    async fn send(&self, msg: Message) -> Result<(), SendError>;
    
    /// 发送系统消息
    async fn send_system(&self, msg: SystemMessage) -> Result<(), SendError>;
    
    /// 启动消息处理
    async fn start(&self, ctx: Arc<Context>) -> Result<(), SendError>;
    
    /// 停止消息处理
    async fn stop(&self) -> Result<(), SendError>;
    
    /// 暂停消息处理
    async fn suspend(&self) -> Result<(), SendError>;
    
    /// 恢复消息处理
    async fn resume(&self) -> Result<(), SendError>;
    
    /// 获取当前状态
    fn status(&self) -> MailboxStatus;
    
    /// 获取当前消息数量
    fn len(&self) -> usize;
    
    /// 是否为空
    fn is_empty(&self) -> bool;
    
    /// 接收一条消息
    async fn receive(&self) -> Result<Option<Message>, SendError>;
    
    /// 获取调度器
    fn dispatcher(&self) -> Arc<dyn MailboxDispatcher>;
    
    /// 获取指标
    fn metrics(&self) -> Arc<MailboxMetrics>;
    
    /// 设置调度器
    fn set_dispatcher(&self, dispatcher: Arc<dyn MailboxDispatcher>);
    
    /// 获取配置
    fn config(&self) -> &MailboxConfig;
    
    /// 接收指定优先级的消息
    async fn receive_priority(&self, priority: MessagePriority) -> Result<Option<Message>, SendError> {
        self.receive().await
    }
    
    /// 批量接收消息
    async fn receive_batch(&self, max_size: usize) -> Result<Vec<Message>, SendError> {
        let mut batch = Vec::with_capacity(max_size);
        for _ in 0..max_size {
            match self.receive().await? {
                Some(msg) => batch.push(msg),
                None => break,
            }
        }
        Ok(batch)
    }
    
    /// 批量发送消息
    async fn send_batch(&self, messages: Vec<Message>) -> Result<(), SendError> {
        for msg in messages {
            self.send(msg).await?;
        }
        Ok(())
    }
    
    /// 清空邮箱
    async fn clear(&self) -> Result<(), SendError>;
    
    /// 获取邮箱统计信息
    fn stats(&self) -> MailboxStats;
    
    /// 设置邮箱配置
    fn set_config(&mut self, config: MailboxConfig);
}

/// 消息处理器
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, ctx: &Context, msg: Message) -> Result<(), SendError>;
}

/// 系统消息处理器
#[async_trait::async_trait]
pub trait SystemMessageHandler: Send + Sync {
    async fn handle(&self, ctx: &Context, msg: SystemMessage) -> Result<(), SendError>;
}

pub enum MailboxKind {
    Unbounded,
    Bounded(usize),
    Priority,
}

impl MailboxKind {
    pub fn create(&self, config: MailboxConfig) -> Box<dyn Mailbox> {
        match self {
            MailboxKind::Unbounded => Box::new(UnboundedMailbox::new(config)),
            MailboxKind::Bounded(capacity) => {
                let mut config = config;
                config.capacity = *capacity;
                Box::new(BoundedMailbox::new(config))
            }
            MailboxKind::Priority => Box::new(PriorityMailbox::new(config)),
        }
    }
} 