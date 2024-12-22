mod actor_ref;
mod basic_actors;
mod lifecycle;
mod decorators;
mod props;

pub use actor_ref::ActorRef;
pub use basic_actors::{ForwardActor, BatchActor};
pub use lifecycle::{ActorLifecycle, LifecycleAware};
pub use decorators::{ThrottleDecorator, RetryDecorator};
pub use props::Props;

use async_trait::async_trait;
use crate::{Context, Message, SystemMessage};

#[async_trait]
pub trait Actor: Send + 'static {
    /// Called when actor is started
    async fn started(&mut self, ctx: &mut Context) {}
    
    /// Called when actor is stopped
    async fn stopped(&mut self, ctx: &mut Context) {}
    
    /// Handle incoming message
    async fn receive(&mut self, ctx: &mut Context, msg: Message);

    /// Handle system messages
    async fn system_receive(&mut self, ctx: &mut Context, msg: SystemMessage) {
        match msg {
            SystemMessage::Stop => {
                self.stopped(ctx).await;
                ctx.stop();
            }
            SystemMessage::Restart => {
                self.stopped(ctx).await;
                self.started(ctx).await;
            }
            _ => {}
        }
    }
}

pub trait ActorFactory<A: Actor> {
    fn create(&self) -> A;
}

impl<F, A> ActorFactory<A> for F 
where
    F: Fn() -> A,
    A: Actor,
{
    fn create(&self) -> A {
        (self)()
    }
} 