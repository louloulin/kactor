use std::sync::Arc;
use crate::{Actor, Context, Message, Pid, SendError, Props};
use dashmap::DashMap;

pub mod strategies;
pub use strategies::*;

#[derive(Debug, Clone, Copy)]
pub enum RouterKind {
    Broadcast,    // 广播到所有 routee
    Random,       // 随机选择 routee
    RoundRobin,   // 轮询选择 routee
    Consistent,   // 一致性哈希
    SmallestMailbox, // 选择邮箱最小的 routee
}

pub struct Router {
    kind: RouterKind,
    routees: Arc<DashMap<String, Pid>>,
    strategy: Box<dyn RoutingStrategy>,
}

#[async_trait::async_trait]
pub trait RoutingStrategy: Send + Sync {
    async fn route(&self, message: &Message, routees: &DashMap<String, Pid>) -> Vec<Pid>;
    fn add_routee(&mut self, pid: Pid);
    fn remove_routee(&mut self, pid: &Pid);
}

impl Router {
    pub fn new(kind: RouterKind) -> Self {
        let strategy: Box<dyn RoutingStrategy> = match kind {
            RouterKind::Broadcast => Box::new(BroadcastStrategy::new()),
            RouterKind::Random => Box::new(RandomStrategy::new()),
            RouterKind::RoundRobin => Box::new(RoundRobinStrategy::new()),
            RouterKind::Consistent => Box::new(ConsistentHashStrategy::new()),
            RouterKind::SmallestMailbox => Box::new(SmallestMailboxStrategy::new()),
        };

        Self {
            kind,
            routees: Arc::new(DashMap::new()),
            strategy,
        }
    }

    pub fn with_routees(mut self, routees: Vec<Pid>) -> Self {
        for routee in routees {
            self.add_routee(routee);
        }
        self
    }

    pub fn add_routee(&mut self, pid: Pid) {
        self.routees.insert(pid.id.clone(), pid.clone());
        self.strategy.add_routee(pid);
    }

    pub fn remove_routee(&mut self, pid: &Pid) {
        self.routees.remove(&pid.id);
        self.strategy.remove_routee(pid);
    }

    pub async fn route(&self, message: &Message) -> Result<(), SendError> {
        let targets = self.strategy.route(message, &self.routees).await;
        
        match message {
            Message::BroadcastMessage(msg) => {
                for target in targets {
                    if let Err(err) = target.send(msg.as_ref().clone()).await {
                        log::error!("Failed to broadcast message to {}: {:?}", target.id, err);
                    }
                }
                Ok(())
            }
            Message::RouteMessage { message, .. } => {
                if let Some(target) = targets.first() {
                    target.send(message.as_ref().clone()).await
                } else {
                    Err(SendError::NoRoutee)
                }
            }
            _ => {
                if let Some(target) = targets.first() {
                    target.send(message.clone()).await
                } else {
                    Err(SendError::NoRoutee)
                }
            }
        }
    }

    pub fn get_routees(&self) -> Vec<Pid> {
        self.routees.iter().map(|entry| entry.value().clone()).collect()
    }

    pub fn routee_count(&self) -> usize {
        self.routees.len()
    }
} 