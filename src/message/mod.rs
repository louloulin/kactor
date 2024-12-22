mod envelope;
mod system_message;

pub use envelope::MessageEnvelope;
pub use system_message::SystemMessage;

use std::any::Any;
use crate::Pid;

#[derive(Clone)]
pub struct Message {
    pub payload: Box<dyn Any + Send>,
    pub sender: Option<Pid>,
    pub header: Option<MessageHeader>,
}

impl Message {
    pub fn new<T: Any + Send>(payload: T) -> Self {
        Self {
            payload: Box::new(payload),
            sender: None,
            header: None,
        }
    }

    pub fn with_sender(mut self, sender: Pid) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn with_header(mut self, header: MessageHeader) -> Self {
        self.header = Some(header);
        self
    }

    pub fn downcast<T: Any>(&self) -> Result<&T, &dyn Any> {
        self.payload.downcast_ref::<T>().ok_or(&*self.payload)
    }
}

#[derive(Default, Clone)]
pub struct MessageHeader {
    headers: dashmap::DashMap<String, String>,
}

impl MessageHeader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.headers.get(key).map(|v| v.clone())
    }

    pub fn set(&self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn remove(&self, key: &str) -> Option<String> {
        self.headers.remove(key).map(|(_, v)| v)
    }

    pub fn clear(&self) {
        self.headers.clear();
    }
} 