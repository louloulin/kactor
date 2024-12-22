use async_trait::async_trait;
use std::error::Error;
use crate::{Actor, Context, Message};
use super::{Event, Snapshot, Recovery};
use super::errors::PersistenceError;

#[async_trait]
pub trait PersistentActor: Actor {
    type State: Send + Sync;
    type Event: Send + Sync;
    
    fn persistence_id(&self) -> String;
    
    async fn recover(&mut self, ctx: &mut Context, recovery: Recovery) -> Result<(), Box<dyn Error>>;
    
    async fn persist_event(&mut self, ctx: &mut Context, event: Event) -> Result<(), Box<dyn Error>>;
    
    async fn persist_snapshot(
        &mut self,
        ctx: &mut Context,
        state: Self::State,
    ) -> Result<(), Box<dyn Error>>;
    
    fn update_state(&mut self, event: &Self::Event);
    
    fn get_state(&self) -> &Self::State;
}

pub struct Persistence {
    event_store: Arc<dyn EventStore>,
    snapshot_store: Arc<dyn SnapshotStore>,
    recovery_strategy: RecoveryStrategy,
}

impl Persistence {
    pub fn new(
        event_store: Arc<dyn EventStore>,
        snapshot_store: Arc<dyn SnapshotStore>,
        recovery_strategy: RecoveryStrategy,
    ) -> Self {
        Self {
            event_store,
            snapshot_store,
            recovery_strategy,
        }
    }

    pub async fn recover<A: PersistentActor>(
        &self,
        actor: &mut A,
        ctx: &mut Context,
    ) -> Result<(), Box<dyn Error>> {
        let persistence_id = actor.persistence_id();
        
        // 获取最新快照
        let snapshot = self.snapshot_store
            .get_latest_snapshot(&persistence_id)
            .await?;
        
        // 获取事件
        let from_sequence_nr = snapshot
            .as_ref()
            .map(|s| s.sequence_nr + 1)
            .unwrap_or(0);
            
        let to_sequence_nr = self.event_store
            .get_highest_sequence_nr(&persistence_id)
            .await?;
            
        let events = self.event_store
            .get_events(&persistence_id, from_sequence_nr, to_sequence_nr)
            .await?;
            
        let recovery = Recovery {
            snapshot,
            events,
        };
        
        actor.recover(ctx, recovery).await
    }

    pub async fn persist_event<A: PersistentActor>(
        &self,
        actor: &mut A,
        ctx: &mut Context,
        event: A::Event,
    ) -> Result<(), PersistenceError> {
        let persistence_id = actor.persistence_id();
        let sequence_nr = self.event_store
            .get_highest_sequence_nr(&persistence_id)
            .await
            .map_err(|e| PersistenceError::EventStore(e.to_string()))? + 1;

        // 序列化事件
        let event_data = bincode::serialize(&event)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let event = Event {
            persistence_id: persistence_id.clone(),
            sequence_nr,
            payload: event_data,
            manifest: std::any::type_name::<A::Event>().to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            metadata: None,
        };

        // 持久化事件
        self.event_store
            .persist_event(event)
            .await
            .map_err(|e| PersistenceError::EventStore(e.to_string()))?;

        // 更新 actor 状态
        actor.update_state(&event);

        // 检查是否需要创建快照
        if let Some(interval) = self.recovery_strategy.snapshot_interval {
            if sequence_nr % (interval.as_secs() as u64) == 0 {
                self.create_snapshot(actor, ctx).await?;
            }
        }

        Ok(())
    }

    pub async fn create_snapshot<A: PersistentActor>(
        &self,
        actor: &mut A,
        ctx: &mut Context,
    ) -> Result<(), PersistenceError> {
        let persistence_id = actor.persistence_id();
        let sequence_nr = self.event_store
            .get_highest_sequence_nr(&persistence_id)
            .await
            .map_err(|e| PersistenceError::EventStore(e.to_string()))?;

        // 序列化状态
        let state = actor.get_state();
        let state_data = bincode::serialize(state)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let snapshot = Snapshot {
            persistence_id: persistence_id.clone(),
            sequence_nr,
            state: state_data,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: None,
        };

        // 保存快照
        self.snapshot_store
            .save_snapshot(snapshot)
            .await
            .map_err(|e| PersistenceError::SnapshotStore(e.to_string()))?;

        // 清理旧快照
        self.cleanup_old_snapshots(&persistence_id).await?;

        Ok(())
    }

    async fn cleanup_old_snapshots(&self, persistence_id: &str) -> Result<(), PersistenceError> {
        let snapshots = self.snapshot_store
            .get_snapshots(persistence_id)
            .await
            .map_err(|e| PersistenceError::SnapshotStore(e.to_string()))?;

        // 保留最新的N个快照
        const MAX_SNAPSHOTS: usize = 5;
        if snapshots.len() > MAX_SNAPSHOTS {
            let oldest_to_keep = snapshots[snapshots.len() - MAX_SNAPSHOTS].sequence_nr;
            self.snapshot_store
                .delete_snapshots(persistence_id, oldest_to_keep)
                .await
                .map_err(|e| PersistenceError::SnapshotStore(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn delete_events(
        &self,
        persistence_id: &str,
        to_sequence_nr: u64,
    ) -> Result<(), PersistenceError> {
        self.event_store
            .delete_events(persistence_id, to_sequence_nr)
            .await
            .map_err(|e| PersistenceError::EventStore(e.to_string()))
    }
} 