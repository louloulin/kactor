use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub persistence_id: String,
    pub sequence_nr: u64,
    pub payload: Vec<u8>,
    pub manifest: String,
    pub timestamp: i64,
    pub metadata: Option<Vec<u8>>,
}

#[async_trait]
pub trait EventStore: Send + Sync {
    async fn persist_event(&self, event: Event) -> Result<(), Box<dyn Error>>;
    
    async fn get_events(
        &self, 
        persistence_id: &str,
        from_sequence_nr: u64,
        to_sequence_nr: u64,
    ) -> Result<Vec<Event>, Box<dyn Error>>;
    
    async fn get_highest_sequence_nr(&self, persistence_id: &str) -> Result<u64, Box<dyn Error>>;
}

// 内存事件存储实现
pub struct InMemoryEventStore {
    events: dashmap::DashMap<String, Vec<Event>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: dashmap::DashMap::new(),
        }
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn persist_event(&self, event: Event) -> Result<(), Box<dyn Error>> {
        self.events
            .entry(event.persistence_id.clone())
            .or_default()
            .push(event);
        Ok(())
    }

    async fn get_events(
        &self,
        persistence_id: &str,
        from_sequence_nr: u64,
        to_sequence_nr: u64,
    ) -> Result<Vec<Event>, Box<dyn Error>> {
        Ok(self
            .events
            .get(persistence_id)
            .map(|events| {
                events
                    .iter()
                    .filter(|e| {
                        e.sequence_nr >= from_sequence_nr && e.sequence_nr <= to_sequence_nr
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn get_highest_sequence_nr(&self, persistence_id: &str) -> Result<u64, Box<dyn Error>> {
        Ok(self
            .events
            .get(persistence_id)
            .map(|events| {
                events
                    .iter()
                    .map(|e| e.sequence_nr)
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0))
    }
} 