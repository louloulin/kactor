use std::collections::BinaryHeap;
use std::cmp::Ordering;
use tokio::sync::mpsc;
use crate::SendError;
use super::Message;

/// A message queue for actors
pub struct MessageQueue {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    capacity: usize,
    queue: BinaryHeap<Message>, // Use a binary heap for priority queue
}

impl MessageQueue {
    /// Creates a new message queue with the given capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver,
            capacity,
            queue: BinaryHeap::new(),
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

    /// Adds a message to the queue, maintaining priority order
    pub fn enqueue(&mut self, msg: Message) -> Result<(), SendError> {
        if self.queue.len() >= self.capacity {
            return Err(SendError::MailboxFull);
        }
        self.queue.push(msg);
        Ok(())
    }

    /// Dequeues the highest priority message
    pub fn dequeue(&mut self) -> Option<Message> {
        self.queue.pop()
    }
}

// Implement Ord and PartialOrd for Message to allow priority queueing
impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority) // Higher priority first
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for Message {} 