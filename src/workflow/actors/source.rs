use crate::actor::{Actor, Context};
use crate::message::Message;
use crate::workflow::SourceConfig;
use async_trait::async_trait;

#[derive(Debug)]
pub struct SourceActor {
    pub name: String,
    pub config: SourceConfig,
}

impl SourceActor {
    pub fn new(name: String, config: SourceConfig) -> Self {
        Self { name, config }
    }
}

#[async_trait]
impl Actor for SourceActor {
    async fn receive(&self, ctx: &Context, msg: Message) {
        // 实现源数据读取和发送逻辑
        // 1. 根据配置读取数据
        // 2. 转换为消息
        // 3. 发送给下游
    }
} 