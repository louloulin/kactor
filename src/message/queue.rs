use tokio::sync::mpsc;
use super::Message;

/// A message queue for actors
pub struct MessageQueue {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    capacity: usize,
}

impl MessageQueue {
    /// Creates a new message queue with the given capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver,
            capacity,
        }
    }

    /// Returns the sender half of the queue
    pub fn sender(&self) -> mpsc::Sender<Message> {
        self.sender.clone()
    }

    /// Returns the receiver half of the queue
    pub fn receiver(&mut self) -> &mut mpsc::Receiver<Message> {
        &mut self.receiver
    }

    /// Returns the capacity of the queue
    pub fn capacity(&self) -> usize {
        self.capacity
    }
} 