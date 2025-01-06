use std::sync::Arc;
use tokio::sync::mpsc;
use crate::message::Message;
use crate::errors::SendError;
use rand::Rng;

/// ActorRef represents a reference to an actor that can receive messages
#[derive(Clone)]
pub struct ActorRef {
    /// The unique identifier of this actor
    id: String,
    /// The sender half of the actor's message channel
    sender: Arc<mpsc::Sender<Message>>,
}

impl ActorRef {
    /// Creates a new ActorRef with a specified ID
    pub fn new(id: String, sender: mpsc::Sender<Message>) -> Self {
        Self {
            id,
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

    /// Default constructor for ActorRef
    pub fn default() -> Self {
        let random_id: String = rand::thread_rng()
            .gen_range(1000..9999)
            .to_string();
        let (sender, _receiver) = mpsc::channel(100); // Specify a buffer size, e.g., 100
        Self {
            id: random_id,
            sender: Arc::new(sender),
        }
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

impl Default for ActorRef {
    fn default() -> Self {
        let random_id: String = rand::thread_rng()
            .gen_range(1000..9999)
            .to_string();
        let (sender, _receiver) = mpsc::channel(100);
        Self {
            id: random_id,
            sender: Arc::new(sender),
        }
    }
}