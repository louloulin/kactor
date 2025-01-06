use std::sync::Arc;
use crate::actor::{Actor, ActorRef, MockActor};
use crate::context::Context;
use crate::dispatcher::{Dispatcher, ThreadPoolDispatcher};
use crate::mailbox::Mailbox;
use crate::errors::SendError;
use crate::supervision::{SupervisorStrategy, DefaultStrategy};

pub struct Props {
    producer: Box<dyn Fn() -> Box<dyn Actor> + Send + Sync>,
    dispatcher: Arc<dyn Dispatcher>,
    supervisor_strategy: Box<dyn SupervisorStrategy>,
    mailbox_size: usize,
}

impl Props {
    pub fn new<F>(producer: F) -> Self 
    where
        F: Fn() -> Box<dyn Actor> + Send + Sync + 'static,
    {
        Self {
            producer: Box::new(producer),
            dispatcher: Arc::new(ThreadPoolDispatcher::default()),
            supervisor_strategy: Box::new(DefaultStrategy::default()),
            mailbox_size: 1000,
        }
    }

    pub fn with_dispatcher(mut self, dispatcher: Arc<dyn Dispatcher>) -> Self {
        self.dispatcher = dispatcher;
        self
    }

    pub fn with_supervisor(mut self, strategy: Box<dyn SupervisorStrategy>) -> Self {
        self.supervisor_strategy = strategy;
        self
    }

    pub fn with_mailbox_size(mut self, size: usize) -> Self {
        self.mailbox_size = size;
        self
    }

    pub fn spawn(&self, parent: Option<ActorRef>) -> Result<ActorRef, SendError> {
        let actor = (self.producer)();
        let mailbox = Mailbox::new(self.mailbox_size);
        
        let context = Context::new(
            ActorRef::default(),
            parent,
            self.dispatcher.clone(),
            self.supervisor_strategy.clone(),
            mailbox.clone(),
        );

        let actor_ref = ActorRef::default();
        context.set_actor_ref(actor_ref.clone());
        
        // 启动 Actor
        self.dispatcher.schedule(async move {
            if let Err(e) = context.start().await {
                log::error!("Failed to start actor: {}", e);
            }
        });

        Ok(actor_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::MockActor; // 假设有一个 MockActor 用于测试

    #[tokio::test]
    async fn test_props_spawn() {
        let props = Props::new(|| Box::new(MockActor::new()));
        let pid = props.spawn(None).unwrap();
        assert!(pid.is_valid()); // 验证 PID 是否有效
    }
} 