use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use crate::{Message, Pid, SendError, RemoteError};

pub struct BatchConfig {
    pub max_batch_size: usize,
    pub max_batch_delay: Duration,
    pub initial_buffer_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            max_batch_delay: Duration::from_millis(50),
            initial_buffer_size: 1024,
        }
    }
}

pub struct BatchManager {
    config: BatchConfig,
    batches: HashMap<String, MessageBatch>,
    sender: mpsc::UnboundedSender<(String, Vec<Message>)>,
}

impl BatchManager {
    pub fn new(config: BatchConfig) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        // 启动批处理任务
        tokio::spawn(Self::process_batches(receiver));
        
        Self {
            config,
            batches: HashMap::new(),
            sender,
        }
    }

    pub async fn add_message(&mut self, target: &Pid, message: Message) -> Result<(), SendError> {
        let batch = self.batches
            .entry(target.address.clone())
            .or_insert_with(|| MessageBatch::new(
                self.config.max_batch_size,
                self.config.max_batch_delay,
                self.config.initial_buffer_size,
            ));

        batch.add_message(message);

        if batch.should_flush() {
            self.flush_batch(&target.address).await?;
        }

        Ok(())
    }

    async fn flush_batch(&mut self, address: &str) -> Result<(), SendError> {
        if let Some(batch) = self.batches.get_mut(address) {
            let messages = batch.take_messages();
            if !messages.is_empty() {
                self.sender.send((address.to_string(), messages))
                    .map_err(|_| SendError::BatchProcessingError)?;
            }
        }
        Ok(())
    }

    async fn process_batches(mut receiver: mpsc::UnboundedReceiver<(String, Vec<Message>)>) {
        while let Some((address, messages)) = receiver.recv().await {
            // 处理批量消息
            if let Err(e) = Self::send_batch(&address, messages).await {
                log::error!("Failed to send batch to {}: {:?}", address, e);
            }
        }
    }

    async fn send_batch(address: &str, messages: Vec<Message>) -> Result<(), RemoteError> {
        // 这里实现批量发送��辑
        Ok(())
    }
}

struct MessageBatch {
    messages: Vec<Message>,
    max_size: usize,
    max_delay: Duration,
    created_at: Instant,
}

impl MessageBatch {
    fn new(max_size: usize, max_delay: Duration, initial_capacity: usize) -> Self {
        Self {
            messages: Vec::with_capacity(initial_capacity),
            max_size,
            max_delay,
            created_at: Instant::now(),
        }
    }

    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    fn should_flush(&self) -> bool {
        self.messages.len() >= self.max_size || 
        self.created_at.elapsed() >= self.max_delay
    }

    fn take_messages(&mut self) -> Vec<Message> {
        let messages = std::mem::take(&mut self.messages);
        self.created_at = Instant::now();
        messages
    }
} 