use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub persistence_id: String,
    pub sequence_nr: u64,
    pub state: Vec<u8>,
    pub timestamp: i64,
    pub metadata: Option<Vec<u8>>,
}

#[async_trait]
pub trait SnapshotStore: Send + Sync {
    async fn save_snapshot(&self, snapshot: Snapshot) -> Result<(), Box<dyn Error>>;
    
    async fn get_latest_snapshot(
        &self,
        persistence_id: &str,
    ) -> Result<Option<Snapshot>, Box<dyn Error>>;
    
    async fn delete_snapshots(
        &self,
        persistence_id: &str,
        to_sequence_nr: u64,
    ) -> Result<(), Box<dyn Error>>;
}

// 内存快照存储实现
pub struct InMemorySnapshotStore {
    snapshots: dashmap::DashMap<String, Vec<Snapshot>>,
}

impl InMemorySnapshotStore {
    pub fn new() -> Self {
        Self {
            snapshots: dashmap::DashMap::new(),
        }
    }
}

#[async_trait]
impl SnapshotStore for InMemorySnapshotStore {
    async fn save_snapshot(&self, snapshot: Snapshot) -> Result<(), Box<dyn Error>> {
        self.snapshots
            .entry(snapshot.persistence_id.clone())
            .or_default()
            .push(snapshot);
        Ok(())
    }

    async fn get_latest_snapshot(
        &self,
        persistence_id: &str,
    ) -> Result<Option<Snapshot>, Box<dyn Error>> {
        Ok(self
            .snapshots
            .get(persistence_id)
            .and_then(|snapshots| {
                snapshots
                    .iter()
                    .max_by_key(|s| s.sequence_nr)
                    .cloned()
            }))
    }

    async fn delete_snapshots(
        &self,
        persistence_id: &str,
        to_sequence_nr: u64,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(mut snapshots) = self.snapshots.get_mut(persistence_id) {
            snapshots.retain(|s| s.sequence_nr > to_sequence_nr);
        }
        Ok(())
    }
} 