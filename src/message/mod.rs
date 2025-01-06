mod batch;
mod envelope;
mod queue;
mod system_message;

use std::any::Any;
use crate::actor::ActorRef;

pub use batch::MessageBatch;
pub use envelope::Envelope;
pub use queue::MessageQueue;
pub use system_message::SystemMessage;
use crate::SendError;

/// Represents a message that can be sent to an actor
#[derive(Clone)]
pub struct Message {
    /// The actual message payload
    pub payload: Box<dyn Any + Send>,
    /// The sender of this message
    pub sender: Option<ActorRef>,
    /// Message headers/metadata
    pub header: Option<Box<dyn Any + Send>>,
    /// The priority of the message (higher value means higher priority)
    pub priority: u8,
}

impl Message {
    /// Creates a new message
    pub fn new<T: Any + Send>(payload: T) -> Self {
        Self {
            payload: Box::new(payload),
            sender: None,
            header: None,
            priority: 0,
        }
    }

    /// Creates a new message with a sender
    pub fn with_sender<T: Any + Send>(payload: T, sender: ActorRef) -> Self {
        Self {
            payload: Box::new(payload),
            sender: Some(sender),
            header: None,
            priority: 0,
        }
    }

    /// Creates a system stop message
    pub(crate) fn system_stop() -> Self {
        Self::new(SystemMessage::Stop)
    }

    /// Sends the message to the specified actor and handles potential errors
    pub async fn send_to(&self, target: &ActorRef) -> Result<(), SendError> {
        target.send(self.clone()).await.map_err(|_| SendError::MailboxFull)
    }

    /// Creates a new message with a specified priority
    pub fn new_with_priority<T: Any + Send>(payload: T, priority: u8) -> Self {
        Self {
            payload: Box::new(payload),
            sender: None,
            header: None,
            priority,
        }
    }
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("sender", &self.sender)
            .field("has_header", &self.header.is_some())
            .finish()
    }
} 