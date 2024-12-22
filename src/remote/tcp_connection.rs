use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::{Message, Pid, SendError, RemoteError, Connection, MessageSerializer};
use crate::remote::{MessageCompressor, FlowController};
use crate::remote::batch::{BatchManager, BatchConfig};

pub struct TcpConnection {
    stream: TcpStream,
    serializer: MessageSerializer,
    compressor: Box<dyn MessageCompressor>,
    flow_controller: FlowController,
    batch_manager: BatchManager,
}

impl TcpConnection {
    pub fn new(
        stream: TcpStream,
        compressor: Box<dyn MessageCompressor>,
        messages_per_sec: usize,
        bytes_per_sec: usize,
        batch_config: BatchConfig,
    ) -> Self {
        Self {
            stream,
            serializer: MessageSerializer::new(),
            compressor,
            flow_controller: FlowController::new(messages_per_sec, bytes_per_sec),
            batch_manager: BatchManager::new(batch_config),
        }
    }

    pub async fn handle(&mut self) {
        while let Ok(message) = self.receive().await {
            // 处理接收到的消息
            if let Err(e) = self.handle_message(message).await {
                log::error!("Error handling message: {:?}", e);
                break;
            }
        }
    }

    async fn handle_message(&mut self, message: Message) -> Result<(), RemoteError> {
        // 将消息转发给本地 actor 系统
        Ok(())
    }

    async fn read_message(&mut self) -> Result<Message, RemoteError> {
        // 读取压缩数据的长度
        let mut len_bytes = [0u8; 4];
        self.stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        // 读取压缩数据
        let mut buffer = vec![0u8; len];
        self.stream.read_exact(&mut buffer).await?;

        // 解压数据
        let decompressed = self.compressor.decompress(&buffer).await?;

        // 反序列化消息
        self.serializer.deserialize(&decompressed).await
    }

    async fn write_message(&mut self, data: &[u8]) -> Result<(), RemoteError> {
        // 检查流量限制
        if !self.flow_controller.can_send(data.len()).await {
            return Err(RemoteError::RateLimitExceeded);
        }

        // 压缩数据
        let compressed = self.compressor.compress(data).await?;

        // 写入压缩后的长度
        let len = compressed.len() as u32;
        self.stream.write_all(&len.to_be_bytes()).await?;

        // 写入压缩后的数据
        self.stream.write_all(&compressed).await?;
        self.stream.flush().await?;

        Ok(())
    }

    async fn write_batch(&mut self, messages: &[Message]) -> Result<(), RemoteError> {
        // 序列化整个批次
        let batch_data = self.serializer.serialize_batch(messages).await?;

        // 检查流量限制
        if !self.flow_controller.can_send(batch_data.len()).await {
            return Err(RemoteError::RateLimitExceeded);
        }

        // 压缩数据
        let compressed = self.compressor.compress(&batch_data).await?;

        // 写入压缩后的长度和数据
        let len = compressed.len() as u32;
        self.stream.write_all(&len.to_be_bytes()).await?;
        self.stream.write_all(&compressed).await?;
        self.stream.flush().await?;

        Ok(())
    }
}

#[async_trait]
impl Connection for TcpConnection {
    async fn send(&mut self, target: &Pid, message: Message) -> Result<(), SendError> {
        // 添加到批处理管理器
        self.batch_manager.add_message(target, message).await
    }

    async fn receive(&mut self) -> Result<Message, RemoteError> {
        self.read_message().await
    }
} 