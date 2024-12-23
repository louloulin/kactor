use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtoError {
    #[error("Send error: {0}")]
    SendError(#[from] SendError),
    
    #[error("Spawn error: {0}")]
    SpawnError(#[from] SpawnError),
    
    #[error("System error: {0}")]
    SystemError(String),
}

#[derive(Debug, Clone, Error)]
pub enum SendError {
    #[error("Dead letter")]
    DeadLetter,
    
    #[error("Mailbox full")]
    MailboxFull,
    
    #[error("System shutting down")]
    SystemShuttingDown,
}

#[derive(Debug, Error)]
pub enum SpawnError {
    #[error("Actor panicked")]
    ActorPanicked,
    
    #[error("System shutting down")]
    SystemShuttingDown,
    
    #[error("Invalid props")]
    InvalidProps,
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

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid state")]
    InvalidState,

    #[error("Unknown actor type: {0}")]
    UnknownActorType(String),

    #[error("Workflow not found: {0}")]
    NotFound(String),

    #[error("Step failed: {0}")]
    StepFailed(String),

    #[error("Mailbox error: {0}")]
    SendError(#[from] SendError),
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