use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct ProtoMetrics {
    providers: Vec<Box<dyn MetricsProvider>>,
    actor_stats: Arc<RwLock<ActorStats>>,
}

impl ProtoMetrics {
    pub fn new(providers: Vec<Box<dyn MetricsProvider>>) -> Self {
        Self {
            providers,
            actor_stats: Arc::new(RwLock::new(ActorStats::default())),
        }
    }

    pub async fn record_message_received(&self, actor_type: &str) {
        let mut stats = self.actor_stats.write().await;
        stats.messages_received += 1;
        
        for provider in &self.providers {
            provider.record_message(actor_type, "received");
        }
    }

    pub async fn record_actor_spawned(&self, actor_type: &str) {
        let mut stats = self.actor_stats.write().await;
        stats.actors_spawned += 1;
        
        for provider in &self.providers {
            provider.record_actor_spawned(actor_type);
        }
    }
}

#[async_trait::async_trait]
pub trait MetricsProvider: Send + Sync {
    async fn record_message(&self, actor_type: &str, message_type: &str);
    async fn record_actor_spawned(&self, actor_type: &str);
    async fn record_actor_stopped(&self, actor_type: &str);
}

#[derive(Default)]
pub struct ActorStats {
    pub messages_received: u64,
    pub messages_sent: u64,
    pub actors_spawned: u64,
    pub actors_stopped: u64,
} 