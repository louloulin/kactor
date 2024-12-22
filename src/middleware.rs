use async_trait::async_trait;
use crate::{Context, Message, SendError};

#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn handle(&self, ctx: &mut Context, msg: Message, next: Next<'_>) -> Result<(), SendError>;
}

pub struct Next<'a> {
    pub(crate) middleware: &'a [Box<dyn Middleware>],
    pub(crate) actor: &'a mut Box<dyn Actor>,
}

impl<'a> Next<'a> {
    pub async fn run(self, ctx: &mut Context, msg: Message) -> Result<(), SendError> {
        if let Some((current, rest)) = self.middleware.split_first() {
            current.handle(
                ctx,
                msg,
                Next {
                    middleware: rest,
                    actor: self.actor,
                },
            ).await
        } else {
            self.actor.receive(ctx, msg).await;
            Ok(())
        }
    }
} 