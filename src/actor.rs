use async_trait::async_trait;
use std::any::Any;
use crate::{Context, Message, SendError};

#[async_trait]
pub trait Actor: Send + 'static {
    // 核心消息处理
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError>;

    // 生命周期钩子
    async fn started(&mut self, ctx: &Context) -> Result<(), SendError> {
        Ok(())
    }

    async fn stopping(&mut self, ctx: &Context) -> Result<(), SendError> {
        Ok(())
    }

    async fn stopped(&mut self, ctx: &Context) -> Result<(), SendError> {
        Ok(())
    }

    async fn restarting(&mut self, ctx: &Context, reason: &str) -> Result<(), SendError> {
        Ok(())
    }

    // 监督策略
    fn supervisor_strategy(&self) -> Box<dyn SupervisorStrategy> {
        Box::new(DefaultStrategy::default())
    }
}

// Actor 引用
#[derive(Clone)]
pub struct ActorRef {
    pid: Pid,
    mailbox: Arc<Mailbox>,
}

impl ActorRef {
    pub async fn send(&self, message: Message) -> Result<(), SendError> {
        self.mailbox.send(message).await
    }

    pub async fn send_system(&self, message: SystemMessage) -> Result<(), SendError> {
        self.mailbox.send_system(message).await
    }

    pub fn stop(&self) {
        self.mailbox.stop();
    }
}

pub struct Context {
    actor: Box<dyn Actor>,
    system: Arc<ActorSystem>,
    self_pid: Pid,
    parent: Option<Pid>,
    children: HashSet<Pid>,
    watchers: HashSet<Pid>,
} 