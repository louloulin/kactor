use crate::actor::{Actor, Context};
use crate::message::Message;
use crate::workflow::SinkConfig;
use async_trait::async_trait;

#[derive(Debug)]
pub struct SinkActor {
    pub name: String,
    pub sink_config: SinkConfig,
}

impl SinkActor {
    pub fn new(name: String, sink_config: SinkConfig) -> Self {
        Self { name, sink_config }
    }
}

#[async_trait]
impl Actor for SinkActor {
    async fn receive(&self, ctx: &Context, msg: Message) {
        // 实现数据写入逻辑
        // 1. 从消息中提取数据
        // 2. 根据配置写入目标系统
        // 3. 处理重试逻辑
    }
} 