use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::{Message, MessageHeader, Pid};

#[async_trait]
pub trait Serializer: Send + Sync {
    async fn serialize(&self, message: &Message) -> Result<Vec<u8>, SerializationError>;
    async fn deserialize(&self, bytes: &[u8]) -> Result<Message, SerializationError>;
}

#[derive(Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub target: Pid,
    pub sender: Option<Pid>,
    pub message_type: String,
    pub message_data: Vec<u8>,
    pub header: Option<MessageHeader>,
}

pub struct MessageSerializer {
    serializers: DashMap<String, Box<dyn Serializer>>,
}

impl MessageSerializer {
    pub fn new() -> Self {
        Self {
            serializers: DashMap::new(),
        }
    }

    pub fn register<S: Serializer + 'static>(&self, message_type: &str, serializer: S) {
        self.serializers.insert(message_type.to_string(), Box::new(serializer));
    }

    pub async fn serialize(&self, message: &Message, target: &Pid) -> Result<Vec<u8>, SerializationError> {
        let message_type = message.payload.type_id().to_string();
        if let Some(serializer) = self.serializers.get(&message_type) {
            let message_data = serializer.serialize(message).await?;
            let envelope = MessageEnvelope {
                target: target.clone(),
                sender: message.sender.clone(),
                message_type,
                message_data,
                header: message.header.clone(),
            };
            bincode::serialize(&envelope).map_err(|e| SerializationError::EncodingError(e.to_string()))
        } else {
            Err(SerializationError::NoSerializerFound(message_type))
        }
    }

    pub async fn serialize_batch(&self, messages: &[Message]) -> Result<Vec<u8>, SerializationError> {
        let mut batch_data = Vec::with_capacity(messages.len() * 100); // 估计大小
        
        // 写入消息数量
        let count = messages.len() as u32;
        batch_data.extend_from_slice(&count.to_be_bytes());

        // 序列化每条消息
        for message in messages {
            let message_data = self.serialize(message, &message.target).await?;
            let len = message_data.len() as u32;
            batch_data.extend_from_slice(&len.to_be_bytes());
            batch_data.extend_from_slice(&message_data);
        }

        Ok(batch_data)
    }

    pub async fn deserialize_batch(&self, data: &[u8]) -> Result<Vec<Message>, SerializationError> {
        let mut messages = Vec::new();
        let mut offset = 0;

        // 读取消息数量
        let count = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap());
        offset += 4;

        // 反序列化每条消息
        for _ in 0..count {
            let len = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            
            let message_data = &data[offset..offset + len];
            offset += len;
            
            let message = self.deserialize(message_data).await?;
            messages.push(message);
        }

        Ok(messages)
    }
} 