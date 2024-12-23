use std::sync::Arc;
use super::Actor;
use crate::middleware::Middleware;
use crate::supervisor::SupervisorStrategy;

pub struct Props {
    // Actor 创建器
    actor_producer: Box<dyn Fn() -> Box<dyn Actor> + Send + Sync>,
    
    // 中间件
    middleware: Vec<Box<dyn Middleware>>,
    
    // 监督策略
    supervisor_strategy: Option<Box<dyn SupervisorStrategy>>,
    
    // 调度器配置
    dispatcher_id: String,
    
    // 邮箱配置
    mailbox_size: usize,
}

impl Props {
    pub fn new<A, F>(producer: F) -> Self 
    where
        A: Actor + 'static,
        F: Fn() -> A + Send + Sync + 'static,
    {
        Self {
            actor_producer: Box::new(move || Box::new(producer())),
            middleware: Vec::new(),
            supervisor_strategy: None,
            dispatcher_id: "default".to_string(),
            mailbox_size: 1000,
        }
    }

    pub fn with_middleware(mut self, middleware: Box<dyn Middleware>) -> Self {
        self.middleware.push(middleware);
        self
    }

    pub fn with_supervisor(mut self, strategy: Box<dyn SupervisorStrategy>) -> Self {
        self.supervisor_strategy = Some(strategy);
        self
    }

    pub fn with_dispatcher(mut self, dispatcher_id: impl Into<String>) -> Self {
        self.dispatcher_id = dispatcher_id.into();
        self
    }

    pub fn with_mailbox_size(mut self, size: usize) -> Self {
        self.mailbox_size = size;
        self
    }

    pub(crate) fn create_actor(&self) -> Box<dyn Actor> {
        (self.actor_producer)()
    }

    pub(crate) fn get_middleware(&self) -> &[Box<dyn Middleware>] {
        &self.middleware
    }

    pub(crate) fn get_supervisor(&self) -> Option<&Box<dyn SupervisorStrategy>> {
        self.supervisor_strategy.as_ref()
    }
} 