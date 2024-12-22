mod event_store;
mod snapshot_store;
mod persistent_actor;
mod recovery;

pub use event_store::{EventStore, Event};
pub use snapshot_store::{SnapshotStore, Snapshot};
pub use persistent_actor::{PersistentActor, Persistence};
pub use recovery::{Recovery, RecoveryStrategy};

use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::{Actor, Context, Message}; 

#[async_trait]
pub trait Journal {
    async fn write_events(&self, pid: &str, events: Vec<Event>) -> Result<(), PersistenceError>;
    async fn read_events(&self, pid: &str) -> Result<Vec<Event>, PersistenceError>;
}

#[async_trait]
pub trait Snapshot {
    async fn write_snapshot(&self, pid: &str, state: State) -> Result<(), PersistenceError>;
    async fn read_snapshot(&self, pid: &str) -> Result<Option<State>, PersistenceError>;
}

pub struct Persistence {
    journal: Box<dyn Journal>,
    snapshot: Box<dyn Snapshot>,
    snapshot_interval: usize,
}

impl Persistence {
    pub fn new(
        journal: Box<dyn Journal>,
        snapshot: Box<dyn Snapshot>,
        snapshot_interval: usize,
    ) -> Self {
        Self {
            journal,
            snapshot,
            snapshot_interval,
        }
    }

    pub async fn persist_event(&self, pid: &str, event: Event) -> Result<(), PersistenceError> {
        self.journal.write_events(pid, vec![event]).await
    }

    pub async fn recover_state(&self, pid: &str) -> Result<State, PersistenceError> {
        if let Some(state) = self.snapshot.read_snapshot(pid).await? {
            Ok(state)
        } else {
            let events = self.journal.read_events(pid).await?;
            // 重放事件
            Ok(State::default())
        }
    }
} 