use crate::actor::ActorRef;
use crate::errors::SendError;
use crate::message::Message;
use crate::system::ActorSystem;
use std::sync::Arc;

pub struct RootContext {
    system: Arc<ActorSystem>,
}

impl RootContext {
    pub fn new(system: Arc<ActorSystem>) -> Self {
        Self { system }
    }

    pub async fn send(&self, target: &ActorRef, msg: Message) -> Result<(), SendError> {
        target.send(msg).await
    }
} 