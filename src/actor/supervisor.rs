use async_trait::async_trait;
use crate::actor::Actor;
use crate::context::Context;
use crate::errors::SendError;

#[async_trait]
pub trait SupervisorStrategy: Send + Sync {
    async fn handle_failure(&self, ctx: &mut Context, error: SendError) -> Result<(), SendError>;
} 