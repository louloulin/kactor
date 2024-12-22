mod actor_process;
mod dead_letter;

pub use actor_process::ActorProcess;
pub use dead_letter::DeadLetterProcess;

use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::RwLock;
use crate::{Message, SystemMessage, SendError, Context, Props, Actor};

pub struct Pid {
    pub address: String,
    pub id: String,
}

impl Clone for Pid {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
            id: self.id.clone(),
        }
    }
}

pub struct Process {
    context: Context,
    props: Props,
}

impl Process {
    pub fn new(context: Context, props: Props) -> Self {
        Self { context, props }
    }

    pub async fn start(&self) {
        self.context.start().await;
    }

    pub async fn stop(&self) {
        self.context.stop().await;
    }

    pub async fn send_user_message(&self, message: Message) -> Result<(), SendError> {
        if self.context.is_stopping() {
            return Err(SendError::DeadLetter);
        }
        self.context.receive_message(message).await
    }

    pub async fn send_system_message(&self, message: SystemMessage) -> Result<(), SendError> {
        if self.context.is_stopping() {
            return Err(SendError::DeadLetter);
        }
        self.context.receive_system_message(message).await
    }
}

pub struct ProcessRegistry {
    processes: DashMap<String, Arc<Process>>,
    sequence_id: RwLock<u64>,
}

impl ProcessRegistry {
    pub fn new() -> Self {
        Self {
            processes: DashMap::new(),
            sequence_id: RwLock::new(0),
        }
    }

    pub async fn next_pid(&self) -> Pid {
        let id = {
            let mut seq = self.sequence_id.write().await;
            *seq += 1;
            *seq
        };
        Pid {
            address: "local".to_string(),
            id: format!("{}${}", "local", id),
        }
    }

    pub async fn add(&self, pid: Pid, process: Process) -> Result<(), SpawnError> {
        if self.processes.contains_key(&pid.id) {
            return Err(SpawnError::DuplicatePid);
        }
        self.processes.insert(pid.id, Arc::new(process));
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<Arc<Process>> {
        self.processes.get(id).map(|p| Arc::clone(p))
    }

    pub fn remove(&self, id: &str) {
        self.processes.remove(id);
    }

    pub fn get_all(&self) -> Vec<Pid> {
        self.processes
            .iter()
            .map(|entry| Pid {
                address: "local".to_string(),
                id: entry.key().clone(),
            })
            .collect()
    }
}

#[async_trait::async_trait]
pub trait Process: Send + Sync {
    async fn send_message(&self, message: Message) -> Result<(), SendError>;
    fn pid(&self) -> Pid;
} 