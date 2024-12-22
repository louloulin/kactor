use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::{Actor, Context, Message, SendError, Pid};

/// 工作流状态
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowState {
    Created,
    Running,
    Paused,
    Completed,
    Failed,
}

/// 工作流步骤
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    /// 执行步骤
    async fn execute(&self, ctx: &Context) -> Result<(), WorkflowError>;
    
    /// 获取步骤名称
    fn name(&self) -> &str;
    
    /// 获取依赖步骤
    fn dependencies(&self) -> Vec<String>;
    
    /// 回滚步骤
    async fn rollback(&self, ctx: &Context) -> Result<(), WorkflowError>;
}

/// 工作流定义
pub struct Workflow {
    name: String,
    steps: Vec<Box<dyn WorkflowStep>>,
    state: Arc<RwLock<WorkflowState>>,
    current_step: Arc<RwLock<usize>>,
}

impl Workflow {
    pub fn new(name: String) -> Self {
        Self {
            name,
            steps: Vec::new(),
            state: Arc::new(RwLock::new(WorkflowState::Created)),
            current_step: Arc::new(RwLock::new(0)),
        }
    }

    pub fn add_step<S: WorkflowStep + 'static>(&mut self, step: S) {
        self.steps.push(Box::new(step));
    }

    pub async fn start(&self, ctx: &Context) -> Result<(), WorkflowError> {
        *self.state.write().await = WorkflowState::Running;
        self.execute_next(ctx).await
    }

    pub async fn pause(&self) -> Result<(), WorkflowError> {
        *self.state.write().await = WorkflowState::Paused;
        Ok(())
    }

    pub async fn resume(&self, ctx: &Context) -> Result<(), WorkflowError> {
        *self.state.write().await = WorkflowState::Running;
        self.execute_next(ctx).await
    }

    async fn execute_next(&self, ctx: &Context) -> Result<(), WorkflowError> {
        let mut current = self.current_step.write().await;
        
        while *current < self.steps.len() {
            let step = &self.steps[*current];
            
            // 检查依赖是否满足
            for dep in step.dependencies() {
                if !self.is_step_completed(&dep).await? {
                    return Err(WorkflowError::DependencyNotMet(dep));
                }
            }
            
            // 执行步骤
            if let Err(e) = step.execute(ctx).await {
                *self.state.write().await = WorkflowState::Failed;
                return Err(e);
            }
            
            *current += 1;
        }
        
        *self.state.write().await = WorkflowState::Completed;
        Ok(())
    }

    async fn is_step_completed(&self, step_name: &str) -> Result<bool, WorkflowError> {
        let current = *self.current_step.read().await;
        Ok(self.steps[..current].iter().any(|s| s.name() == step_name))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Step failed: {0}")]
    StepFailed(String),
    
    #[error("Dependency not met: {0}")]
    DependencyNotMet(String),
    
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    #[error("Invalid workflow state")]
    InvalidState,
    
    #[error("Cycle detected in workflow")]
    CycleDetected,
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
} 