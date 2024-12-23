use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct WorkflowContext {
    // 工作流实例信息
    pub workflow_id: String,
    pub parent_workflow_id: Option<String>,
    
    // 共享数据存储
    shared_data: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
    
    // 步骤间通信通道
    channels: Arc<RwLock<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>,
}

impl WorkflowContext {
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            parent_workflow_id: None,
            shared_data: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set_data<T: Any + Send + Sync>(&self, data: T) {
        let mut store = self.shared_data.write().await;
        store.insert(TypeId::of::<T>(), Box::new(data));
    }

    pub async fn get_data<T: Any + Send + Sync>(&self) -> Option<T> {
        let store = self.shared_data.read().await;
        store.get(&TypeId::of::<T>())
            .and_then(|data| data.downcast_ref::<T>())
            .cloned()
    }

    pub async fn create_channel(&self, name: &str) -> tokio::sync::mpsc::UnboundedReceiver<Message> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.channels.write().await.insert(name.to_string(), tx);
        rx
    }

    pub async fn send_to_channel(&self, name: &str, msg: Message) -> Result<(), SendError> {
        if let Some(tx) = self.channels.read().await.get(name) {
            tx.send(msg).map_err(|_| SendError::MailboxFull)
        } else {
            Err(SendError::Other("Channel not found".to_string()))
        }
    }
} 