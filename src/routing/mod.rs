use std::sync::Arc;
use dashmap::DashMap;
use crate::{Actor, Context, Message, Pid, SendError};

pub enum RoutingStrategy {
    RoundRobin,
    Random,
    ConsistentHash,
    LeastBusy,
    Broadcast,
}

pub struct Router {
    strategy: RoutingStrategy,
    routees: Arc<DashMap<String, Pid>>,
    state: Arc<RouterState>,
}

impl Router {
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            strategy,
            routees: Arc::new(DashMap::new()),
            state: Arc::new(RouterState::new()),
        }
    }

    pub fn add_routee(&self, pid: Pid) {
        self.routees.insert(pid.id.clone(), pid);
    }

    pub fn remove_routee(&self, pid: &Pid) {
        self.routees.remove(&pid.id);
    }

    pub async fn route(&self, msg: Message) -> Result<(), SendError> {
        match self.strategy {
            RoutingStrategy::RoundRobin => {
                self.route_round_robin(msg).await
            }
            RoutingStrategy::Random => {
                self.route_random(msg).await
            }
            RoutingStrategy::ConsistentHash => {
                self.route_consistent_hash(msg).await
            }
            RoutingStrategy::LeastBusy => {
                self.route_least_busy(msg).await
            }
            RoutingStrategy::Broadcast => {
                self.route_broadcast(msg).await
            }
        }
    }

    async fn route_round_robin(&self, msg: Message) -> Result<(), SendError> {
        let next = self.state.next_round_robin(self.routees.len());
        if let Some(routee) = self.routees.iter().nth(next) {
            routee.send(msg).await
        } else {
            Err(SendError::NoRoutee)
        }
    }

    async fn route_random(&self, msg: Message) -> Result<(), SendError> {
        use rand::seq::SliceRandom;
        let routees: Vec<_> = self.routees.iter().collect();
        if let Some(routee) = routees.choose(&mut rand::thread_rng()) {
            routee.send(msg).await
        } else {
            Err(SendError::NoRoutee)
        }
    }

    async fn route_consistent_hash(&self, msg: Message) -> Result<(), SendError> {
        let hash = msg.hash();
        let routees: Vec<_> = self.routees.iter().collect();
        let idx = hash as usize % routees.len();
        if let Some(routee) = routees.get(idx) {
            routee.send(msg).await
        } else {
            Err(SendError::NoRoutee)
        }
    }

    async fn route_least_busy(&self, msg: Message) -> Result<(), SendError> {
        let least_busy = self.state.get_least_busy_routee(&self.routees);
        if let Some(routee) = least_busy {
            routee.send(msg).await
        } else {
            Err(SendError::NoRoutee)
        }
    }

    async fn route_broadcast(&self, msg: Message) -> Result<(), SendError> {
        let mut errors = Vec::new();
        for routee in self.routees.iter() {
            if let Err(e) = routee.send(msg.clone()).await {
                errors.push(e);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(SendError::BroadcastFailed(errors))
        }
    }
} 