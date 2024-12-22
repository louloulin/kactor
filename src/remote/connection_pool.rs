use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;
use crate::{Connection, RemoteError, SendError};

pub struct ConnectionPool {
    connections: DashMap<String, Vec<Arc<RwLock<Box<dyn Connection>>>>>,
    max_connections: usize,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: DashMap::new(),
            max_connections,
        }
    }

    pub async fn get_connection(&self, address: &str) -> Result<Arc<RwLock<Box<dyn Connection>>>, RemoteError> {
        if let Some(mut conns) = self.connections.get_mut(address) {
            // 简单的轮询策略
            if let Some(conn) = conns.pop() {
                conns.insert(0, Arc::clone(&conn));
                return Ok(conn);
            }
        }
        Err(RemoteError::ConnectionError("No available connection".to_string()))
    }

    pub async fn add_connection(&self, address: String, connection: Box<dyn Connection>) {
        let conn = Arc::new(RwLock::new(connection));
        self.connections
            .entry(address)
            .or_insert_with(Vec::new)
            .push(conn);
    }

    pub async fn remove_connection(&self, address: &str, index: usize) {
        if let Some(mut conns) = self.connections.get_mut(address) {
            if index < conns.len() {
                conns.remove(index);
            }
        }
    }
} 