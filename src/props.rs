use std::sync::Arc;
use crate::{Actor, Context, Dispatcher, Mailbox, Pid, SendError, SupervisorStrategy};

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

    pub fn spawn(&self, parent: Option<Pid>) -> Result<Pid, SendError> {
        let actor = (self.producer)();
        let mailbox = Mailbox::new(self.mailbox_size);
        
        let context = Context::new(
            actor,
            parent,
            self.dispatcher.clone(),
            self.supervisor_strategy.clone(),
            mailbox.clone(),
        );

        let pid = Pid::new();
        context.set_pid(pid.clone());
        
        // 启动 Actor
        self.dispatcher.schedule(async move {
            if let Err(e) = context.start().await {
                log::error!("Failed to start actor: {}", e);
            }
        });

        Ok(pid)
    }
} 