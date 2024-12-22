use super::*;
use std::collections::VecDeque;
use std::sync::RwLock;

pub struct MessageQueue<T> {
    queue: RwLock<VecDeque<T>>,
    capacity: usize,
}

impl<T> MessageQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn push(&self, msg: T) -> Result<(), SendError> {
        let mut queue = self.queue.write().unwrap();
        if queue.len() >= self.capacity {
            return Err(SendError::MailboxFull);
        }
        queue.push_back(msg);
        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        self.queue.write().unwrap().pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.read().unwrap().is_empty()
    }

    pub fn clear(&self) {
        self.queue.write().unwrap().clear();
    }
}

impl<T> Default for MessageQueue<T> {
    fn default() -> Self {
        Self::new(1000)
    }
} 