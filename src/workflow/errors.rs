use std::time::Duration;
use thiserror::Error;
use crate::workflow::state::NodeState;

#[derive(Debug, Error)]
pub enum StepError {
    #[error("Step execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Step timeout after {0:?}")]
    Timeout(Duration),

    #[error("Step cancelled")]
    Cancelled,

    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: NodeState,
        to: NodeState,
    },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<StepError> for WorkflowError {
    fn from(err: StepError) -> Self {
        WorkflowError::StepFailed(err.to_string())
    }
} 