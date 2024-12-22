use std::sync::Arc;
use crate::{ActorSystem, Message, MessageEnvelope, RemoteError};

pub struct RemoteMessageHandler {
    system: Arc<ActorSystem>,
}

impl RemoteMessageHandler {
    pub fn new(system: Arc<ActorSystem>) -> Self {
        Self { system }
    }

    pub async fn handle_envelope(&self, envelope: MessageEnvelope) -> Result<(), RemoteError> {
        let message = Message {
            payload: envelope.message_data,
            sender: envelope.sender,
            header: envelope.header,
        };

        self.system.send(&envelope.target, message).await
            .map_err(|e| RemoteError::ConnectionError(format!("Failed to deliver message: {:?}", e)))
    }

    pub async fn handle_system_message(&self, message: SystemMessage) -> Result<(), RemoteError> {
        match message {
            SystemMessage::Watch(watcher, target) => {
                // 处理监视请求
                if let Some(process) = self.system.process_registry.get(&target.id) {
                    process.watch(watcher).await;
                }
            }
            SystemMessage::Unwatch(watcher, target) => {
                // 处理取消监视请求
                if let Some(process) = self.system.process_registry.get(&target.id) {
                    process.unwatch(&watcher).await;
                }
            }
            SystemMessage::Terminated(pid) => {
                // 处理终止通知
                self.system.process_registry.remove(&pid.id);
            }
            _ => {}
        }
        Ok(())
    }
}