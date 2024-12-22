use tokio::sync::mpsc;
use crate::{Message, SystemMessage};

pub enum QueueMessage {
    User(Message),
    System(SystemMessage),
}

pub struct MessageQueue {
    sender: mpsc::UnboundedSender<QueueMessage>,
    receiver: mpsc::UnboundedReceiver<QueueMessage>,
}

impl MessageQueue {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<QueueMessage> {
        self.sender.clone()
    }

    pub async fn receive(&mut self) -> Option<QueueMessage> {
        self.receiver.recv().await
    }

    pub fn try_send(&self, msg: QueueMessage) -> Result<(), mpsc::error::SendError<QueueMessage>> {
        self.sender.send(msg)
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
} 