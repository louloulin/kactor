use std::sync::Arc;
use tokio::sync::broadcast;
use std::any::Any;

pub struct EventStream {
    sender: broadcast::Sender<Box<dyn Any + Send>>,
}

impl EventStream {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self { sender }
    }

    pub fn subscribe<T: 'static>(&self) -> broadcast::Receiver<Box<dyn Any + Send>> {
        self.sender.subscribe()
    }

    pub fn publish<T: Any + Send>(&self, event: T) {
        let _ = self.sender.send(Box::new(event));
    }
}

pub struct Subscription {
    _receiver: broadcast::Receiver<Box<dyn Any + Send>>,
} 