use crate::actor::Actor;
use crate::context::Context;
use crate::message::Message;
use crate::errors::SendError;

pub struct MockActor;

#[async_trait::async_trait]
impl Actor for MockActor {
    async fn receive(&mut self, _ctx: &Context, _msg: Message) -> Result<(), SendError> {
        // Mock implementation
        Ok(())
    }
} 