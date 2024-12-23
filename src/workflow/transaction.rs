use super::*;

pub struct WorkflowTransaction {
    steps: Vec<Box<dyn WorkflowStep>>,
    compensations: Vec<Box<dyn WorkflowStep>>,
    state: TransactionState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    Active,
    Committed,
    RollingBack,
    RolledBack,
}

impl WorkflowTransaction {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            compensations: Vec::new(),
            state: TransactionState::Active,
        }
    }

    pub fn add_step<S: WorkflowStep + 'static>(&mut self, step: S, compensation: S) {
        self.steps.push(Box::new(step));
        self.compensations.push(Box::new(compensation));
    }

    pub async fn execute(&mut self, ctx: &Context) -> Result<(), WorkflowError> {
        for step in &self.steps {
            if let Err(e) = step.execute(ctx).await {
                self.rollback(ctx).await?;
                return Err(e);
            }
        }
        self.state = TransactionState::Committed;
        Ok(())
    }

    pub async fn rollback(&mut self, ctx: &Context) -> Result<(), WorkflowError> {
        self.state = TransactionState::RollingBack;
        for compensation in self.compensations.iter().rev() {
            compensation.execute(ctx).await?;
        }
        self.state = TransactionState::RolledBack;
        Ok(())
    }
} 