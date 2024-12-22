use async_trait::async_trait;
use std::sync::Arc;
use crate::{Actor, Context, Message, SendError};

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle<A: Actor>(
        &self,
        ctx: &mut Context,
        msg: Message,
        next: Arc<dyn Fn(Message) -> Result<(), SendError> + Send + Sync>,
    ) -> Result<(), SendError>;
}

pub struct MiddlewareChain {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add<M: Middleware + 'static>(&mut self, middleware: M) {
        self.middlewares.push(Box::new(middleware));
    }

    pub async fn execute(
        &self,
        ctx: &mut Context,
        msg: Message,
    ) -> Result<(), SendError> {
        let mut chain = self.middlewares.iter();
        
        let next = move |msg: Message| {
            if let Some(middleware) = chain.next() {
                middleware.handle(ctx, msg, Arc::new(next))
            } else {
                ctx.actor.receive(ctx, msg)
            }
        };

        next(msg).await
    }
} 