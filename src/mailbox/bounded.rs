use super::*;
use tokio::sync::mpsc;
use std::sync::RwLock;
use std::time::Duration;

pub struct BoundedMailbox {
    config: MailboxConfig,
    status: Arc<RwLock<MailboxStatus>>,
    sender: mpsc::Sender<Message>,
    receiver: Arc<RwLock<Option<mpsc::Receiver<Message>>>>,
    system_sender: mpsc::UnboundedSender<SystemMessage>,
    system_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SystemMessage>>>>,
}

impl BoundedMailbox {
    pub fn new(config: MailboxConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.capacity);
        let (system_tx, system_rx) = mpsc::unbounded_channel();
        
        Self {
            config,
            status: Arc::new(RwLock::new(MailboxStatus::Open)),
            sender: tx,
            receiver: Arc::new(RwLock::new(Some(rx))),
            system_sender: system_tx,
            system_receiver: Arc::new(RwLock::new(Some(system_rx))),
        }
    }
}

#[async_trait::async_trait]
impl Mailbox for BoundedMailbox {
    async fn send(&self, msg: Message) -> Result<(), SendError> {
        let status = *self.status.read().unwrap();
        if status != MailboxStatus::Open {
            return Err(SendError::MailboxClosed);
        }

        self.sender.send(msg).await
            .map_err(|_| SendError::MailboxFull)
    }

    async fn send_system(&self, msg: SystemMessage) -> Result<(), SendError> {
        self.system_sender.send(msg)
            .map_err(|_| SendError::MailboxClosed)
    }

    async fn start(&self, ctx: Arc<Context>) -> Result<(), SendError> {
        let status = Arc::clone(&self.status);
        let receiver = Arc::clone(&self.receiver);
        let system_receiver = Arc::clone(&self.system_receiver);
        let throughput = self.config.throughput;

        tokio::spawn(async move {
            let mut rx = receiver.write().unwrap().take()
                .expect("Mailbox already started");
            let mut system_rx = system_receiver.write().unwrap().take()
                .expect("Mailbox already started");

            while *status.read().unwrap() == MailboxStatus::Open {
                // 优先处理系统消息
                while let Ok(sys_msg) = system_rx.try_recv() {
                    if let Err(e) = ctx.handle_system(sys_msg).await {
                        log::error!("Failed to handle system message: {:?}", e);
                    }
                }

                // 处理用户消息
                let mut processed = 0;
                while processed < throughput {
                    match rx.try_recv() {
                        Ok(msg) => {
                            if let Err(e) = ctx.handle(msg).await {
                                log::error!("Failed to handle message: {:?}", e);
                            }
                            processed += 1;
                        }
                        Err(_) => break,
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
        // 注意：这只是一个近似值，因为channel不提供精确的长度
        self.sender.capacity().unwrap_or(0)
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn dispatcher(&self) -> Arc<dyn MailboxDispatcher> {
        Arc::clone(&self.config.dispatcher)
    }

    fn metrics(&self) -> Arc<MailboxMetrics> {
        // Return metrics for this mailbox
        Arc::new(MailboxMetrics::new()) // Adjust as needed
    }

    fn set_dispatcher(&mut self, dispatcher: Arc<dyn MailboxDispatcher>) {
        self.config.dispatcher = dispatcher;
    }

    fn config(&self) -> &MailboxConfig {
        &self.config
    }

    fn clear(&mut self) {
        // Clear the mailbox
        self.sender = mpsc::channel(self.config.capacity).0; // Reset the sender
    }

    fn stats(&self) -> MailboxStats {
        MailboxStats {
            messages_processed: 0, // Replace with actual stats
            messages_queued: 0,    // Replace with actual stats
            messages_dropped: 0,   // Replace with actual stats
            avg_processing_time: Duration::new(0, 0), // Replace with actual stats
            avg_queuing_time: Duration::new(0, 0),    // Replace with actual stats
            errors: 0,             // Replace with actual stats
            status_changes: 0,     // Replace with actual stats
        }
    }

    fn set_config(&mut self, config: MailboxConfig) {
        self.config = config;
    }

    async fn receive(&self) -> Result<Option<Message>, SendError> {
        let receiver = self.receiver.read().unwrap();
        if let Some(ref rx) = *receiver {
            match rx.recv().await {
                Some(msg) => Ok(Some(msg)),
                None => Ok(None), // Channel closed
            }
        } else {
            Err(SendError::MailboxClosed)
        }
    }
} 