mod endpoint;
mod serialization;
mod transport;

pub use endpoint::{RemoteEndpoint, EndpointManager};
pub use serialization::{Serializer, MessageSerializer};
pub use transport::{Transport, TcpTransport};

use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;
use crate::{ActorSystem, Message, Pid, SendError};
use crate::remote::message_handler::RemoteMessageHandler;
use crate::remote::errors::RemoteError;
use crate::remote::system_message::SystemMessage;

pub struct RemoteConfig {
    pub host: String,
    pub port: u16,
    pub serializer: Box<dyn Serializer>,
}

pub struct Remote {
    system: Arc<ActorSystem>,
    config: RemoteConfig,
    endpoints: DashMap<String, Arc<RemoteEndpoint>>,
    transport: Arc<RwLock<Box<dyn Transport>>>,
}

impl Remote {
    pub fn new(system: Arc<ActorSystem>, config: RemoteConfig) -> Self {
        Self {
            system,
            config,
            endpoints: DashMap::new(),
            transport: Arc::new(RwLock::new(Box::new(TcpTransport::new()))),
        }
    }

    pub async fn start(&self) -> Result<(), RemoteError> {
        let transport = self.transport.read().await;
        transport.start(&self.config).await
    }

    pub async fn send(&self, target: &Pid, message: Message) -> Result<(), SendError> {
        if let Some(endpoint) = self.endpoints.get(&target.address) {
            endpoint.send(target, message).await
        } else {
            let endpoint = RemoteEndpoint::new(
                target.address.clone(),
                Arc::clone(&self.system),
                Arc::clone(&self.transport),
            );
            self.endpoints.insert(target.address.clone(), Arc::new(endpoint));
            endpoint.send(target, message).await
        }
    }

    pub async fn handle_remote_message(&self, envelope: MessageEnvelope) -> Result<(), RemoteError> {
        let handler = RemoteMessageHandler::new(Arc::clone(&self.system));
        handler.handle_envelope(envelope).await
    }

    pub async fn handle_remote_system_message(&self, message: SystemMessage) -> Result<(), RemoteError> {
        let handler = RemoteMessageHandler::new(Arc::clone(&self.system));
        handler.handle_system_message(message).await
    }

    pub fn endpoint_manager(&self) -> &EndpointManager {
        &self.endpoint_manager
    }
} 