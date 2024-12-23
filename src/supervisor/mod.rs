use async_trait::async_trait;
use crate::context::Context;
use crate::errors::SendError;

#[async_trait]
pub trait SupervisorStrategy: Send + Sync {
    async fn handle_failure(&self, ctx: &mut Context, error: SendError) -> Result<(), SendError>;
}

pub struct OneForOneStrategy {
    max_retries: u32,
    within_time: std::time::Duration,
}

impl OneForOneStrategy {
    pub fn new(max_retries: u32, within_time: std::time::Duration) -> Self {
        Self {
            max_retries,
            within_time,
        }
    }
}

#[async_trait]
impl SupervisorStrategy for OneForOneStrategy {
    async fn handle_failure(&self, ctx: &mut Context, error: SendError) -> Result<(), SendError> {
        // TODO: Implement retry logic
        Ok(())
    }
} 