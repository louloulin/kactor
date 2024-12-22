use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PersistenceError {
    EventStore(String),
    SnapshotStore(String),
    Serialization(String),
    Recovery(String),
    Timeout(String),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersistenceError::EventStore(msg) => write!(f, "Event store error: {}", msg),
            PersistenceError::SnapshotStore(msg) => write!(f, "Snapshot store error: {}", msg),
            PersistenceError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            PersistenceError::Recovery(msg) => write!(f, "Recovery error: {}", msg),
            PersistenceError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
        }
    }
}

impl Error for PersistenceError {} 