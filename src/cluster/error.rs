use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum ClusterError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Member not found: {0}")]
    MemberNotFound(String),

    #[error("Partition error: {0}")]
    Partition(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("State error: {0}")]
    State(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Cluster operation timeout")]
    Timeout,

    #[error("Cluster is shutting down")]
    ShuttingDown,

    #[error("Operation not permitted: {0}")]
    NotPermitted(String),
}

impl From<ClusterError> for SendError {
    fn from(err: ClusterError) -> Self {
        match err {
            ClusterError::Network(_) => SendError::ConnectionFailed,
            ClusterError::Timeout => SendError::Timeout,
            ClusterError::ShuttingDown => SendError::SystemShuttingDown,
            _ => SendError::Other(err.to_string()),
        }
    }
} 