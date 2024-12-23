use std::sync::Arc;
use tokio::sync::mpsc;
use crate::actor::{Actor, ActorRef};
use crate::message::Message;
use crate::errors::SendError;

/// Context provides the Actor with information about its environment and methods to interact with the system
pub struct Context {
    /// Reference to self as an actor
    self_ref: ActorRef,
    /// Channel for sending messages to parent
    parent: Option<ActorRef>,
    /// Channels for child actors
    children: Vec<ActorRef>,
    /// Actor's mailbox sender
    sender: mpsc::Sender<Message>,
    /// Whether the actor is stopping
    stopping: bool,
}

impl Context {
    /// Creates a new Context
    pub fn new(self_ref: ActorRef, parent: Option<ActorRef>, sender: mpsc::Sender<Message>) -> Self {
        Self {
            self_ref,
            parent,
            children: Vec::new(),
            sender,
            stopping: false,
        }
    }

    /// Returns a reference to self as an actor
    pub fn self_ref(&self) -> &ActorRef {
        &self.self_ref
    }

    /// Sends a message to another actor
    pub async fn send(&self, target: &ActorRef, msg: Message) -> Result<(), SendError> {
        target.send(msg).await
    }

    /// Spawns a new child actor
    pub async fn spawn<A: Actor>(&mut self, actor: A) -> Result<ActorRef, SendError> {
        // TODO: Implement actor spawning logic
        todo!()
    }

    /// Stops the actor
    pub async fn stop(&mut self) {
        if !self.stopping {
            self.stopping = true;
            // Stop all children first
            for child in &self.children {
                child.stop().await;
            }
            // Then stop self
            self.self_ref.stop().await;
        }
    }

    /// Returns whether the actor is stopping
    pub fn is_stopping(&self) -> bool {
        self.stopping
    }

    /// Adds a child actor
    pub(crate) fn add_child(&mut self, child: ActorRef) {
        self.children.push(child);
    }

    /// Removes a child actor
    pub(crate) fn remove_child(&mut self, child: &ActorRef) {
        if let Some(pos) = self.children.iter().position(|x| x == child) {
            self.children.swap_remove(pos);
        }
    }
}
