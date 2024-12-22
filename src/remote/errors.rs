use std::error::Error;
use std::fmt;
use tokio::io;

#[derive(Debug)]
pub enum RemoteError {
    IoError(io::Error),
    SerializationError(SerializationError),
    ConnectionError(String),
    HandshakeError(String),
    CompressionError(CompressionError),
    RateLimitExceeded,
}

#[derive(Debug)]
pub enum SerializationError {
    EncodingError(String),
    DecodingError(String),
    NoSerializerFound(String),
}

#[derive(Debug)]
pub enum CompressionError {
    CompressionFailed(String),
    DecompressionFailed(String),
}

impl fmt::Display for RemoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoteError::IoError(e) => write!(f, "IO error: {}", e),
            RemoteError::SerializationError(e) => write!(f, "Serialization error: {:?}", e),
            RemoteError::ConnectionError(e) => write!(f, "Connection error: {}", e),
            RemoteError::HandshakeError(e) => write!(f, "Handshake error: {}", e),
            RemoteError::CompressionError(e) => write!(f, "Compression error: {:?}", e),
            RemoteError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
        }
    }
}

impl Error for RemoteError {}

impl From<io::Error> for RemoteError {
    fn from(error: io::Error) -> Self {
        RemoteError::IoError(error)
    }
}

impl From<SerializationError> for RemoteError {
    fn from(error: SerializationError) -> Self {
        RemoteError::SerializationError(error)
    }
}

impl From<CompressionError> for RemoteError {
    fn from(error: CompressionError) -> Self {
        RemoteError::CompressionError(error)
    }
} 