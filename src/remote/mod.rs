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
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

pub struct RemoteConfig {
    pub host: String,
    pub port: u16,
    pub serializer: Box<dyn Serializer>,
}

pub struct RemoteContext {
    config: RemoteConfig,
    connections: Arc<DashMap<String, RemoteEndpoint>>,
}

impl RemoteContext {
    pub async fn new(config: RemoteConfig) -> Result<Self, std::io::Error> {
        let ctx = Self {
            config,
            connections: Arc::new(DashMap::new()),
        };

        ctx.start_server().await?;
        Ok(ctx)
    }

    pub async fn send(&self, target: &Pid, msg: Message) -> Result<(), SendError> {
        let endpoint = self.get_or_connect_endpoint(&target.address).await?;
        endpoint.send(target, msg).await
    }

    async fn start_server(&self) -> Result<(), std::io::Error> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(addr).await?;

        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                self.handle_connection(stream).await;
            }
        });

        Ok(())
    }
} 