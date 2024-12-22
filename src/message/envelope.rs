use std::any::Any;
use crate::{Message, MessageHeader, Pid};

pub struct MessageEnvelope {
    pub message: Message,
    pub sender: Option<Pid>,
    pub header: Option<MessageHeader>,
    pub target: Pid,
}

impl MessageEnvelope {
    pub fn new(message: Message, target: Pid) -> Self {
        Self {
            message,
            sender: None,
            header: None,
            target,
        }
    }

    pub fn with_sender(mut self, sender: Pid) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn with_header(mut self, header: MessageHeader) -> Self {
        self.header = Some(header);
        self
    }
} 