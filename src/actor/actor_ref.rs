use std::sync::Arc;
use crate::{Message, Pid, SendError};

#[derive(Clone)]
pub struct ActorRef {
    pid: Pid,
    process: Arc<dyn Process>,
}

impl ActorRef {
    pub fn new(pid: Pid, process: Arc<dyn Process>) -> Self {
        Self { pid, process }
    }

    pub async fn send(&self, message: Message) -> Result<(), SendError> {
        self.process.send_message(message).await
    }

    pub fn pid(&self) -> &Pid {
        &self.pid
    }
} 