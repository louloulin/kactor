use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{Message, Pid, SendError, ActorSystem, Transport};
use std::time::Duration;
use connection_pool::ConnectionPool;
use heartbeat::HeartbeatManager;
use reconnection::ReconnectionStrategy;

pub struct RemoteEndpoint {
    address: String,
    system: Arc<ActorSystem>,
    transport: Arc<RwLock<Box<dyn Transport>>>,
    connection_pool: Arc<ConnectionPool>,
    heartbeat_manager: Arc<HeartbeatManager>,
    reconnection_strategy: ReconnectionStrategy,
}

impl RemoteEndpoint {
    pub fn new(
        address: String,
        system: Arc<ActorSystem>,
        transport: Arc<RwLock<Box<dyn Transport>>>,
    ) -> Self {
        Self {
            address,
            system,
            transport,
            connection_pool: Arc::new(ConnectionPool::new(5)),
            heartbeat_manager: Arc::new(HeartbeatManager::new(
                Duration::from_secs(5),
                Duration::from_secs(15),
            )),
            reconnection_strategy: ReconnectionStrategy::new(
                5,
                Duration::from_secs(1),
                Duration::from_secs(30),
            ),
        }
    }

    pub async fn send(&self, target: &Pid, message: Message) -> Result<(), SendError> {
        if let Ok(conn) = self.connection_pool.get_connection(&self.address).await {
            let mut conn = conn.write().await;
            conn.send(target, message).await
        } else {
            // 尝试重连
            self.reconnection_strategy.attempt_reconnect(self).await?;
            self.send(target, message).await
        }
    }

    pub async fn handle_connection_lost(&self) {
        // 通知所有相关的 actor
        // 清理连接池
        // 尝试重连
    }

    pub async fn send_heartbeat(&self, heartbeat: HeartbeatMessage) -> Result<(), SendError> {
        let message = Message::new(heartbeat);
        self.send(&Pid::new(), message).await
    }
}

pub struct EndpointManager {
    endpoints: DashMap<String, Arc<RemoteEndpoint>>,
}

impl EndpointManager {
    pub fn new() -> Self {
        Self {
            endpoints: DashMap::new(),
        }
    }

    pub fn get_or_create(
        &self,
        address: String,
        system: Arc<ActorSystem>,
        transport: Arc<RwLock<Box<dyn Transport>>>,
    ) -> Arc<RemoteEndpoint> {
        if let Some(endpoint) = self.endpoints.get(&address) {
            endpoint.clone()
        } else {
            let endpoint = Arc::new(RemoteEndpoint::new(address.clone(), system, transport));
            self.endpoints.insert(address, Arc::clone(&endpoint));
            endpoint
        }
    }
} 