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

/// Represents a message that can be sent to an actor
#[derive(Clone)]
pub struct Message {
    /// The actual message payload
    pub payload: Box<dyn Any + Send>,
    /// The sender of this message
    pub sender: Option<ActorRef>,
    /// Message headers/metadata
    pub header: Option<Box<dyn Any + Send>>,
}

impl Message {
    /// Creates a new message
    pub fn new<T: Any + Send>(payload: T) -> Self {
        Self {
            payload: Box::new(payload),
            sender: None,
            header: None,
        }
    }

    /// Creates a new message with a sender
    pub fn with_sender<T: Any + Send>(payload: T, sender: ActorRef) -> Self {
        Self {
            payload: Box::new(payload),
            sender: Some(sender),
            header: None,
        }
    }

    /// Creates a system stop message
    pub(crate) fn system_stop() -> Self {
        Self::new(SystemMessage::Stop)
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