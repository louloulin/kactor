use std::any::{Any, TypeId};
use dashmap::DashMap;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use crate::errors::WorkflowError;

pub trait StateBackend: Send + Sync {
    async fn save_state(&self, key: &str, value: Vec<u8>) -> Result<(), WorkflowError>;
    async fn load_state(&self, key: &str) -> Result<Option<Vec<u8>>, WorkflowError>;
    async fn clear_state(&self, key: &str) -> Result<(), WorkflowError>;
}

pub struct StateManager {
    local_state: DashMap<TypeId, Box<dyn Any + Send + Sync>>,
    backend: Arc<dyn StateBackend>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowStatus {
    Created,
    Running,
    Paused,
    Failed,
    Completed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Default for WorkflowStatus {
    fn default() -> Self {
        WorkflowStatus::Created
    }
}

impl Default for NodeState {
    fn default() -> Self {
        NodeState::Pending
    }
} 