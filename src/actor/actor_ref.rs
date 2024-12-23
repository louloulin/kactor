use std::sync::Arc;
use tokio::sync::mpsc;
use crate::message::Message;
use crate::errors::SendError;

/// ActorRef represents a reference to an actor that can receive messages
#[derive(Clone)]
pub struct ActorRef {
    /// The unique identifier of this actor
    id: String,
    /// The sender half of the actor's message channel
    sender: Arc<mpsc::Sender<Message>>,
}

impl ActorRef {
    /// Creates a new ActorRef
    pub fn new(id: impl Into<String>, sender: mpsc::Sender<Message>) -> Self {
        Self {
            id: id.into(),
            sender: Arc::new(sender),
        }
    }

    /// Returns the actor's identifier
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Sends a message to this actor
    pub async fn send(&self, msg: Message) -> Result<(), SendError> {
        self.sender.send(msg).await.map_err(|_| SendError::MailboxFull)
    }

    /// Stops this actor
    pub async fn stop(&self) {
        // Send stop message
        let stop_msg = Message::system_stop();
        let _ = self.send(stop_msg).await;
    }
}

impl std::fmt::Debug for ActorRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorRef")
            .field("id", &self.id)
            .finish()
    }
}

impl PartialEq for ActorRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ActorRef {}

impl std::hash::Hash for ActorRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}