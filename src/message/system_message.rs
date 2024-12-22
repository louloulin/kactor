use crate::{Pid, SendError};

#[derive(Debug, Clone)]
pub enum SystemMessage {
    Stop,
    Restart,
    Resume,
    Suspend,
    Failure(SendError),
    Watch(Pid),
    Unwatch(Pid),
    Terminated(Pid),
} 