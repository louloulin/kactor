mod message_middleware;

use async_trait::async_trait;
use futures::future::BoxFuture;
use crate::context::Context;
use crate::message::Message;
use crate::errors::SendError;

pub use message_middleware::LoggingMiddleware;

pub type Next<'a> = Box<dyn FnOnce(&mut Context, Message) -> BoxFuture<'a, Result<(), SendError>> + Send + 'a>;

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, ctx: &mut Context, msg: Message, next: Next<'_>) -> Result<(), SendError>;
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

    pub async fn execute(&self, ctx: &mut Context, msg: Message) -> Result<(), SendError> {
        fn make_next<'a>(
            chain: &'a mut std::slice::Iter<'a, Box<dyn Middleware>>,
        ) -> Next<'a> {
            Box::new(move |ctx, msg| {
                Box::pin(async move {
                    match chain.next() {
                        Some(middleware) => middleware.handle(ctx, msg, make_next(chain)).await,
                        None => ctx.receive(msg).await
                    }
                })
            })
        }

        let mut chain = self.middlewares.iter();
        make_next(&mut chain)(ctx, msg).await
    }
} 