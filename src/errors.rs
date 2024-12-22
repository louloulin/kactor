use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProtoError {
    SendError(SendError),
    SpawnError(SpawnError),
    SystemError(String),
}

#[derive(Debug, Clone)]
pub enum SendError {
    DeadLetter,
    MailboxFull,
    SystemShuttingDown,
    ConnectionFailed,
    SerializationError,
    NoRemoteAvailable,
    BatchProcessingError,
    NoRoutee,
    Other(String),
}

#[derive(Debug)]
pub enum SpawnError {
    ActorPanicked,
    SystemShuttingDown,
    InvalidProps,
    DuplicatePid,
}

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("No routee available")]
    NoRoutee,

    #[error("Invalid router configuration: {0}")]
    InvalidConfig(String),

    #[error("Routee failed: {0}")]
    RouteeFailed(String),

    #[error("Message routing failed: {0}")]
    RoutingFailed(String),
}

impl Error for ProtoError {}
impl Error for SendError {}
impl Error for SpawnError {}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProtoError::SendError(e) => write!(f, "Send error: {:?}", e),
            ProtoError::SpawnError(e) => write!(f, "Spawn error: {:?}", e),
            ProtoError::SystemError(e) => write!(f, "System error: {}", e),
        }
    }
}

impl From<RouterError> for SendError {
    fn from(err: RouterError) -> Self {
        match err {
            RouterError::NoRoutee => SendError::NoRoutee,
            RouterError::InvalidConfig(msg) => SendError::Other(msg),
            RouterError::RouteeFailed(msg) => SendError::DeadLetter,
            RouterError::RoutingFailed(msg) => SendError::Other(msg),
        }
    }
} 