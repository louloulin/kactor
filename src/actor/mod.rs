mod actor_ref;
mod basic_actors;
mod lifecycle;
mod decorators;
mod props;
mod builder;
mod mock_actor;
pub use actor_ref::ActorRef;
pub use basic_actors::{ForwardActor, BatchActor};
pub use lifecycle::{ActorLifecycle, LifecycleAware};
pub use decorators::{ThrottleDecorator, RetryDecorator};
pub use props::Props;
pub use mock_actor::MockActor;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::MockActor; // 假设有一个 MockActor 用于测试

    #[tokio::test]
    async fn test_actor_ref() {
        let actor_ref = ActorRef::new();
        assert!(actor_ref.is_valid()); // 验证 ActorRef 是否有效
    }
} 