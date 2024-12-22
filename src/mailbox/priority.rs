use super::*;
use priority_queue::PriorityQueue;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    System = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

pub trait PrioritizedMessage {
    fn priority(&self) -> MessagePriority;
}

impl PrioritizedMessage for Message {
    fn priority(&self) -> MessagePriority {
        match self {
            Message::System(_) => MessagePriority::System,
            Message::User(msg) => msg.priority(),
        }
    }
}

pub struct PriorityMailbox {
    config: MailboxConfig,
    status: Arc<RwLock<MailboxStatus>>,
    queue: Arc<RwLock<PriorityQueue<Message, i32>>>,
    system_queue: mpsc::UnboundedSender<SystemMessage>,
}

impl PriorityMailbox {
    pub fn new(config: MailboxConfig) -> Self {
        let (system_tx, _) = mpsc::unbounded_channel();
        
        Self {
            config,
            status: Arc::new(RwLock::new(MailboxStatus::Open)),
            queue: Arc::new(RwLock::new(PriorityQueue::new())),
            system_queue: system_tx,
        }
    }
}

#[async_trait::async_trait]
impl Mailbox for PriorityMailbox {
    async fn send(&self, msg: Message) -> Result<(), SendError> {
        let status = *self.status.read().unwrap();
        if status != MailboxStatus::Open {
            return Err(SendError::MailboxClosed);
        }

        let mut queue = self.queue.write().unwrap();
        if queue.len() >= self.config.capacity {
            return Err(SendError::MailboxFull);
        }

        let priority = msg.priority();
        queue.push(msg, priority);
        Ok(())
    }

    async fn send_system(&self, msg: SystemMessage) -> Result<(), SendError> {
        self.system_queue.send(msg)
            .map_err(|_| SendError::MailboxClosed)
    }

    async fn start(&self, ctx: Arc<Context>) -> Result<(), SendError> {
        let queue = Arc::clone(&self.queue);
        let status = Arc::clone(&self.status);
        let throughput = self.config.throughput;

        tokio::spawn(async move {
            while *status.read().unwrap() == MailboxStatus::Open {
                let mut processed = 0;
                while processed < throughput {
                    let msg = {
                        let mut queue = queue.write().unwrap();
                        queue.pop().map(|(msg, _)| msg)
                    };

                    match msg {
                        Some(msg) => {
                            if let Err(e) = ctx.handle(msg).await {
                                log::error!("Failed to handle message: {:?}", e);
                            }
                            processed += 1;
                        }
                        None => break,
                    }
                }
                tokio::task::yield_now().await;
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<(), SendError> {
        *self.status.write().unwrap() = MailboxStatus::Closed;
        Ok(())
    }

    async fn suspend(&self) -> Result<(), SendError> {
        *self.status.write().unwrap() = MailboxStatus::Suspended;
        Ok(())
    }

    async fn resume(&self) -> Result<(), SendError> {
        *self.status.write().unwrap() = MailboxStatus::Open;
        Ok(())
    }

    fn status(&self) -> MailboxStatus {
        *self.status.read().unwrap()
    }

    fn len(&self) -> usize {
        self.queue.read().unwrap().len()
    }

    fn is_empty(&self) -> bool {
        self.queue.read().unwrap().is_empty()
    }
} 