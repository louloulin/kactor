use std::any::Any;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Message {
    // ... 已有的消息类型 ...
    
    // 路由相关消息
    AddRoutee(Pid),
    RemoveRoutee(Pid),
    GetRoutees,
    BroadcastMessage(Box<Message>),
    RouteMessage {
        message: Box<Message>,
        hash_key: Option<String>,
    },
}

// 为消息添加路由相关的特征
pub trait Routable {
    fn hash_key(&self) -> Option<String>;
}

impl Message {
    pub fn hash_key(&self) -> Option<String> {
        match self {
            Message::RouteMessage { hash_key, .. } => hash_key.clone(),
            _ => None,
        }
    }
}

pub struct MessageHeader {
    headers: HashMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Pid {
    pub address: String,
    pub id: String,
} 