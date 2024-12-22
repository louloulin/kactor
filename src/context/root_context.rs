use std::sync::Arc;
use crate::{ActorSystem, Message, Pid, SendError, Props, Actor, SpawnError};

pub struct RootContext {
    system: Arc<ActorSystem>,
}

impl RootContext {
    pub(crate) fn new(system: Arc<ActorSystem>) -> Self {
        Self { system }
    }

    pub async fn spawn<A: Actor>(&self, props: Props<A>) -> Result<Pid, SpawnError> {
        self.system.spawn(props, None).await
    }

    pub async fn send(&self, pid: &Pid, message: Message) -> Result<(), SendError> {
        self.system.send(pid, message).await
    }

    pub fn stop(&self, pid: &Pid) {
        self.system.stop(pid);
    }

    pub fn system(&self) -> &ActorSystem {
        &self.system
    }
} 