use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use crate::{Context, Message, SendError};

pub(crate) struct BatchProcessor {
    max_batch_size: usize,
    max_wait_time: Duration,
    sender: mpsc::UnboundedSender<(Context, Message)>,
}

impl BatchProcessor {
    pub fn new(max_batch_size: usize, max_wait_time: Duration) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(max_batch_size);
            let mut last_process = Instant::now();

            while let Some((ctx, msg)) = receiver.recv().await {
                batch.push((ctx, msg));

                let should_process = batch.len() >= max_batch_size || 
                    last_process.elapsed() >= max_wait_time;

                if should_process {
                    Self::process_batch(&mut batch).await;
                    last_process = Instant::now();
                }
            }
        });

        Self {
            max_batch_size,
            max_wait_time,
            sender,
        }
    }

    async fn process_batch(batch: &mut Vec<(Context, Message)>) {
        for (ctx, msg) in batch.drain(..) {
            if let Err(e) = ctx.receive_message(msg).await {
                log::error!("Failed to process batched message: {:?}", e);
            }
        }
    }

    pub fn send(&self, ctx: Context, msg: Message) -> Result<(), SendError> {
        self.sender
            .send((ctx, msg))
            .map_err(|_| SendError::MailboxFull)
    }
} 