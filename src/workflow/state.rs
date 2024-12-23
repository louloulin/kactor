use std::any::{Any, TypeId};
use dashmap::DashMap;

pub trait StateBackend: Send + Sync {
    async fn save_state(&self, key: &str, value: Vec<u8>) -> Result<(), WorkflowError>;
    async fn load_state(&self, key: &str) -> Result<Option<Vec<u8>>, WorkflowError>;
    async fn clear_state(&self, key: &str) -> Result<(), WorkflowError>;
}

pub struct WorkflowState {
    local_state: DashMap<TypeId, Box<dyn Any + Send + Sync>>,
    backend: Arc<dyn StateBackend>,
}

impl WorkflowState {
    pub async fn set<T: Serialize + DeserializeOwned + Send + Sync + 'static>(
        &self,
        key: &str,
        value: T,
    ) -> Result<(), WorkflowError> {
        // 保存到本地状态
        self.local_state.insert(TypeId::of::<T>(), Box::new(value.clone()));
        
        // 序列化并保存到后端
        let serialized = bincode::serialize(&value)?;
        self.backend.save_state(key, serialized).await
    }

    pub async fn get<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        key: &str,
    ) -> Result<Option<T>, WorkflowError> {
        // 先查本地缓存
        if let Some(value) = self.local_state.get(&TypeId::of::<T>()) {
            if let Some(typed_value) = value.downcast_ref::<T>() {
                return Ok(Some(typed_value.clone()));
            }
        }
        
        // 从后端加载
        if let Some(data) = self.backend.load_state(key).await? {
            let value: T = bincode::deserialize(&data)?;
            self.local_state.insert(TypeId::of::<T>(), Box::new(value.clone()));
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
} 