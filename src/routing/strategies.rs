use std::sync::atomic::{AtomicUsize, Ordering};
use rand::Rng;
use consistent_hash_ring::ConsistentHashRing;
use dashmap::DashMap;
use crate::{Message, Pid, Context};

pub struct BroadcastStrategy;

impl BroadcastStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl RoutingStrategy for BroadcastStrategy {
    async fn route(&self, _message: &Message, routees: &DashMap<String, Pid>) -> Vec<Pid> {
        routees.iter().map(|entry| entry.value().clone()).collect()
    }

    fn add_routee(&mut self, _pid: Pid) {}
    fn remove_routee(&mut self, _pid: &Pid) {}
}

pub struct RandomStrategy;

impl RandomStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl RoutingStrategy for RandomStrategy {
    async fn route(&self, _message: &Message, routees: &DashMap<String, Pid>) -> Vec<Pid> {
        let routees: Vec<_> = routees.iter().map(|entry| entry.value().clone()).collect();
        if routees.is_empty() {
            return vec![];
        }
        let idx = rand::thread_rng().gen_range(0..routees.len());
        vec![routees[idx].clone()]
    }

    fn add_routee(&mut self, _pid: Pid) {}
    fn remove_routee(&mut self, _pid: &Pid) {}
}

pub struct RoundRobinStrategy {
    current: AtomicUsize,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            current: AtomicUsize::new(0),
        }
    }
}

#[async_trait::async_trait]
impl RoutingStrategy for RoundRobinStrategy {
    async fn route(&self, _message: &Message, routees: &DashMap<String, Pid>) -> Vec<Pid> {
        let routees: Vec<_> = routees.iter().map(|entry| entry.value().clone()).collect();
        if routees.is_empty() {
            return vec![];
        }
        let current = self.current.fetch_add(1, Ordering::SeqCst);
        vec![routees[current % routees.len()].clone()]
    }

    fn add_routee(&mut self, _pid: Pid) {}
    fn remove_routee(&mut self, _pid: &Pid) {}
}

pub struct ConsistentHashStrategy {
    ring: ConsistentHashRing<String>,
}

impl ConsistentHashStrategy {
    pub fn new() -> Self {
        Self {
            ring: ConsistentHashRing::new(),
        }
    }
}

#[async_trait::async_trait]
impl RoutingStrategy for ConsistentHashStrategy {
    async fn route(&self, message: &Message, routees: &DashMap<String, Pid>) -> Vec<Pid> {
        if let Some(key) = message.hash_key() {
            if let Some(node) = self.ring.get_node(&key) {
                if let Some(pid) = routees.get(node) {
                    return vec![pid.clone()];
                }
            }
        }
        vec![]
    }

    fn add_routee(&mut self, pid: Pid) {
        self.ring.add_node(pid.id);
    }

    fn remove_routee(&mut self, pid: &Pid) {
        self.ring.remove_node(&pid.id);
    }
}

pub struct SmallestMailboxStrategy {
    mailbox_sizes: Arc<DashMap<String, usize>>,
}

impl SmallestMailboxStrategy {
    pub fn new() -> Self {
        Self {
            mailbox_sizes: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl RoutingStrategy for SmallestMailboxStrategy {
    async fn route(&self, _message: &Message, routees: &DashMap<String, Pid>) -> Vec<Pid> {
        let mut smallest = None;
        let mut smallest_size = usize::MAX;

        for entry in routees.iter() {
            if let Some(size) = self.mailbox_sizes.get(&entry.key()) {
                if *size < smallest_size {
                    smallest_size = *size;
                    smallest = Some(entry.value().clone());
                }
            }
        }

        smallest.map(|pid| vec![pid]).unwrap_or_default()
    }

    fn add_routee(&mut self, pid: Pid) {
        self.mailbox_sizes.insert(pid.id, 0);
    }

    fn remove_routee(&mut self, pid: &Pid) {
        self.mailbox_sizes.remove(&pid.id);
    }
} 