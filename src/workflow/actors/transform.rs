use crate::actor::{Actor, Context};
use crate::message::Message;
use async_trait::async_trait;

#[derive(Debug)]
pub struct TransformActor {
    pub name: String,
    pub transform_fn: Box<dyn Fn(Message) -> Message + Send + Sync>,
}

impl TransformActor {
    pub fn new(name: String, transform_fn: Box<dyn Fn(Message) -> Message + Send + Sync>) -> Self {
        Self { name, transform_fn }
    }
}

#[async_trait]
impl Actor for TransformActor {
    async fn receive(&self, ctx: &Context, msg: Message) {
        // 实现消息转换逻辑
        let transformed = (self.transform_fn)(msg);
        ctx.send(transformed).await;
    }
} 