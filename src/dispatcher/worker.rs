use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use crate::{Context, Message, SendError};

pub(crate) struct Worker {
    id: usize,
    handle: JoinHandle<()>,
    sender: mpsc::UnboundedSender<WorkerMessage>,
}

enum WorkerMessage {
    Task(Context, Message),
    Shutdown,
}

impl Worker {
    pub fn new(id: usize) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        
        let handle = tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    WorkerMessage::Task(ctx, msg) => {
                        if let Err(e) = ctx.receive_message(msg).await {
                            log::error!("Worker {} failed to process message: {:?}", id, e);
                        }
                    }
                    WorkerMessage::Shutdown => break,
                }
            }
        });

        Self {
            id,
            handle,
            sender,
        }
    }

    pub fn send_task(&self, ctx: Context, msg: Message) -> Result<(), SendError> {
        self.sender
            .send(WorkerMessage::Task(ctx, msg))
            .map_err(|_| SendError::MailboxFull)
    }

    pub async fn shutdown(self) {
        let _ = self.sender.send(WorkerMessage::Shutdown);
        let _ = self.handle.await;
    }
}

pub(crate) struct WorkerPool {
    workers: Vec<Worker>,
}

impl WorkerPool {
    pub fn new(size: usize) -> Self {
        let workers = (0..size)
            .map(|id| Worker::new(id))
            .collect();
        
        Self { workers }
    }

    pub fn get_worker(&self, id: usize) -> Option<&Worker> {
        self.workers.get(id)
    }

    pub async fn shutdown(self) {
        for worker in self.workers {
            worker.shutdown().await;
        }
    }
} 