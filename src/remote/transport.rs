use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};
use crate::{Message, Pid, SendError, RemoteConfig};

#[async_trait]
pub trait Transport: Send + Sync {
    async fn start(&self, config: &RemoteConfig) -> Result<(), RemoteError>;
    async fn connect(&self, address: &str) -> Result<Box<dyn Connection>, SendError>;
}

#[async_trait]
pub trait Connection: Send + Sync {
    async fn send(&mut self, target: &Pid, message: Message) -> Result<(), SendError>;
    async fn receive(&mut self) -> Result<Message, RemoteError>;
}

pub struct TcpTransport {
    listener: Option<TcpListener>,
}

impl TcpTransport {
    pub fn new() -> Self {
        Self { listener: None }
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn start(&self, config: &RemoteConfig) -> Result<(), RemoteError> {
        let addr = format!("{}:{}", config.host, config.port);
        let listener = TcpListener::bind(&addr).await?;
        
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let connection = TcpConnection::new(stream);
                // Handle incoming connection
                tokio::spawn(async move {
                    connection.handle().await;
                });
            }
        });

        Ok(())
    }

    async fn connect(&self, address: &str) -> Result<Box<dyn Connection>, SendError> {
        let stream = TcpStream::connect(address).await
            .map_err(|_| SendError::ConnectionFailed)?;
        Ok(Box::new(TcpConnection::new(stream)))
    }
} 