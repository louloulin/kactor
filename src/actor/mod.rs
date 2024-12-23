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
use crate::context::Context;
use crate::errors::SendError;
use crate::message::Message;

#[async_trait]
pub trait Actor: Send + 'static {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError>;

    async fn started(&mut self, _ctx: &Context) -> Result<(), SendError> {
        Ok(())
    }

    async fn stopping(&mut self, _ctx: &Context) -> Result<(), SendError> {
        Ok(())
    }

    async fn stopped(&mut self, _ctx: &Context) -> Result<(), SendError> {
        Ok(())
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