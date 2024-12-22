use async_trait::async_trait;
use crate::{Message, Pid, Process, SendError, ActorSystem};
use std::sync::Arc;

pub struct DeadLetterProcess {
    system: Arc<ActorSystem>,
}

impl DeadLetterProcess {
    pub(crate) fn new(system: Arc<ActorSystem>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl Process for DeadLetterProcess {
    async fn send_message(&self, message: Message) -> Result<(), SendError> {
        self.system.event_stream.publish(DeadLetterEvent {
            pid: message.sender,
            message,
        });
        Ok(())
    }

    fn pid(&self) -> Pid {
        Pid {
            address: "deadletter".to_string(),
            id: "deadletter".to_string(),
        }
    }
}

pub struct DeadLetterEvent {
    pub pid: Option<Pid>,
    pub message: Message,
} 