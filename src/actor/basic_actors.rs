use async_trait::async_trait;
use futures::future::BoxFuture;
use crate::context::Context;
use crate::message::Message;
use std::collections::HashSet;
use super::{Actor, ActorRef};

/// A simple actor that forwards messages to another actor
pub struct ForwardActor {
    target: ActorRef,
}

#[async_trait]
impl Actor for ForwardActor {
    async fn receive(&mut self, ctx: &mut Context, msg: Message) {
        ctx.send(&self.target, msg).await.ok();
    }
}

/// An actor that aggregates messages and processes them in batches
pub struct BatchActor<T: Send + 'static> {
    batch_size: usize,
    batch: Vec<T>,
    processor: Box<dyn Fn(Vec<T>) -> BoxFuture<'static, ()> + Send + Sync>,
}

#[async_trait]
impl<T: Send + 'static> Actor for BatchActor<T> {
    async fn receive(&mut self, _ctx: &mut Context, msg: Message) {
        if let Ok(item) = msg.payload.downcast::<T>() {
            self.batch.push(*item);
            
            if self.batch.len() >= self.batch_size {
                let batch = std::mem::take(&mut self.batch);
                (self.processor)(batch).await;
            }
        }
    }

    async fn stopped(&mut self, _ctx: &mut Context) {
        if !self.batch.is_empty() {
            let batch = std::mem::take(&mut self.batch);
            (self.processor)(batch).await;
        }
    }
}

/// An actor that broadcasts messages to multiple targets
pub struct BroadcastActor {
    targets: HashSet<ActorRef>,
}

impl BroadcastActor {
    pub fn new() -> Self {
        Self {
            targets: HashSet::new(),
        }
    }

    pub fn add_target(&mut self, target: ActorRef) {
        self.targets.insert(target);
    }

    pub fn remove_target(&mut self, target: &ActorRef) {
        self.targets.remove(target);
    }
}

#[async_trait]
impl Actor for BroadcastActor {
    async fn receive(&mut self, ctx: &mut Context, msg: Message) {
        for target in &self.targets {
            ctx.send(target, msg.clone()).await.ok();
        }
    }
}

/// An actor that filters messages based on a predicate
pub struct FilterActor<T: Send + 'static> {
    target: ActorRef,
    predicate: Box<dyn Fn(&T) -> bool + Send + Sync>,
}

#[async_trait]
impl<T: Send + 'static> Actor for FilterActor<T> {
    async fn receive(&mut self, ctx: &mut Context, msg: Message) {
        if let Ok(payload) = msg.payload.downcast::<T>() {
            if (self.predicate)(&payload) {
                ctx.send(&self.target, msg).await.ok();
            }
        }
    }
}

/// An actor that transforms messages before forwarding them
pub struct TransformActor<T: Send + 'static, U: Send + 'static> {
    target: ActorRef,
    transform: Box<dyn Fn(T) -> U + Send + Sync>,
}

#[async_trait]
impl<T: Send + 'static, U: Send + 'static> Actor for TransformActor<T, U> {
    async fn receive(&mut self, ctx: &mut Context, msg: Message) {
        if let Ok(payload) = msg.payload.downcast::<T>() {
            let transformed = (self.transform)(*payload);
            let new_msg = Message {
                payload: Box::new(transformed),
                sender: msg.sender,
                header: msg.header,
                priority: msg.priority,
            };
            ctx.send(&self.target, new_msg).await.ok();
        }
    }
} 