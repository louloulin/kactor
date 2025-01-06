use std::fmt;

#[derive(Debug)]
pub enum ProtoError {
    MailboxClosed,
    MailboxFull,
    ActorNotFound,
    // 其他错误类型...
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ProtoError {}

#[derive(Debug)]
pub enum SendError {
    DeadLetter,
    MailboxClosed,
    MailboxFull,
    // 其他错误类型...
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SendError {}

#[derive(Debug)]
pub enum SpawnError {
    ActorPanicked,
    InvalidProps,
    // 其他错误类型...
}

impl fmt::Display for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SpawnError {}

#[derive(Debug)]
pub enum WorkflowError {
    Io(std::io::Error),
    Serialization(bincode::Error),
    NodeNotFound(String),
    InvalidState,
    // 其他错误类型...
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for WorkflowError {} 