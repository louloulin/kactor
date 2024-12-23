use crate::errors::SendError;
use crate::actor::ActorRef;

/// System messages used for internal actor communication
#[derive(Debug, Clone)]
pub enum SystemMessage {
    /// Stop the actor
    Stop,
    /// Restart the actor
    Restart,
    /// Resume the actor
    Resume,
    /// Suspend the actor
    Suspend,
    /// Actor failure
    Failure(SendError),
    /// Watch another actor
    Watch(ActorRef),
    /// Unwatch another actor
    Unwatch(ActorRef),
    /// Actor terminated notification
    Terminated(ActorRef),
    /// Actor has been stopped
    Stopped,
    /// Actor has been started
    Started,
    /// Actor is being restarted
    Restarting,
} 