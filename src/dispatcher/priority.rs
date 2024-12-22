use std::cmp::Ordering;
use std::time::Instant;
use tokio::sync::mpsc;
use priority_queue::PriorityQueue;
use crate::{Context, Message, SendError};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Priority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

#[derive(Debug)]
pub struct PrioritizedMessage {
    pub priority: Priority,
    pub timestamp: Instant,
    pub context: Context,
    pub message: Message,
}

impl PartialEq for PrioritizedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl Eq for PrioritizedMessage {}

impl PartialOrd for PrioritizedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => other.timestamp.cmp(&self.timestamp),
            ord => ord,
        }
    }
}

pub struct PriorityDispatcher {
    queue: PriorityQueue<PrioritizedMessage, Priority>,
    sender: mpsc::UnboundedSender<PrioritizedMessage>,
    max_queue_size: usize,
}

impl PriorityDispatcher {
    pub fn new(max_queue_size: usize) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let queue = PriorityQueue::new();

        // 启动处理线程
        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                if let Err(e) = msg.context.receive_message(msg.message).await {
                    log::error!("Failed to process prioritized message: {:?}", e);
                }
            }
        });

        Self {
            queue,
            sender,
            max_queue_size,
        }
    }

    pub fn dispatch_with_priority(
        &mut self,
        priority: Priority,
        ctx: Context,
        msg: Message,
    ) -> Result<(), SendError> {
        if self.queue.len() >= self.max_queue_size {
            return Err(SendError::MailboxFull);
        }

        let prioritized_msg = PrioritizedMessage {
            priority,
            timestamp: Instant::now(),
            context: ctx,
            message: msg,
        };

        self.queue.push(prioritized_msg, priority);
        self.process_queue();
        Ok(())
    }

    fn process_queue(&mut self) {
        while let Some((msg, _)) = self.queue.pop() {
            if let Err(e) = self.sender.send(msg) {
                log::error!("Failed to send prioritized message: {:?}", e);
                break;
            }
        }
    }
} 