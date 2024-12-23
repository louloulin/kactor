use std::fmt;
use std::hash::{Hash, Hasher};

/// Process ID (Pid) represents a unique reference to an actor in the actor system
#[derive(Debug, Clone, Eq)]
pub struct Pid {
    /// The address of the actor system where this actor lives
    pub address: String,
    /// The unique identifier of this actor within its actor system
    pub id: String,
}

impl Pid {
    /// Creates a new Pid with the given address and id
    pub fn new(address: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            id: id.into(),
        }
    }
}

impl PartialEq for Pid {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.id == other.id
    }
}

impl Hash for Pid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.id.hash(state);
    }
}

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.id, self.address)
    }
} 