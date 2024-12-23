use async_trait::async_trait;
use crate::{Actor, Context, Message, SendError, Pid};
use super::{WorkflowStep, WorkflowError};

/// Actor 工作流步骤
#[async_trait]
pub trait ActorWorkflowStep: Actor {
    /// 执行步骤逻辑
    async fn execute_step(&mut self, ctx: &Context) -> Result<(), WorkflowError>;
    
    /// 回滚步骤
    async fn rollback_step(&mut self, ctx: &Context) -> Result<(), WorkflowError>;
}

/// Actor 步骤包装器
pub struct ActorStepWrapper {
    name: String,
    dependencies: Vec<String>,
    actor_pid: Pid,
}

#[async_trait]
impl WorkflowStep for ActorStepWrapper {
    async fn execute(&self, ctx: &Context) -> Result<(), WorkflowError> {
        // 发送执行消息给 Actor
        ctx.send(&self.actor_pid, Message::Execute).await
            .map_err(|e| WorkflowError::StepFailed(format!("Failed to execute step: {}", e)))?;
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn dependencies(&self) -> Vec<String> {
        self.dependencies.clone()
    }

    async fn rollback(&self, ctx: &Context) -> Result<(), WorkflowError> {
        // 发送回���消息给 Actor
        ctx.send(&self.actor_pid, Message::Rollback).await
            .map_err(|e| WorkflowError::StepFailed(format!("Failed to rollback step: {}", e)))?;
        Ok(())
    }
} 