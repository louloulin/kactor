use super::Message;
use crate::actor::ActorRef;

/// An envelope wraps a message with additional metadata
pub struct Envelope {
    /// The wrapped message
    pub message: Message,
    /// The target actor
    pub target: ActorRef,
    /// Whether this is a system message
    pub system: bool,
}

impl Envelope {
    /// Creates a new envelope
    pub fn new(message: Message, target: ActorRef) -> Self {
        Self {
            message,
            target,
            system: false,
        }
    }

    /// Creates a new system message envelope
    pub fn system(message: Message, target: ActorRef) -> Self {
        Self {
            message,
            target,
            system: true,
        }
    }
} 