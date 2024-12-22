use super::*;
use std::collections::HashMap;

pub struct WorkflowExecutor {
    workflows: HashMap<String, Arc<Workflow>>,
    active_workflows: HashMap<String, WorkflowInstance>,
}

pub struct WorkflowInstance {
    workflow: Arc<Workflow>,
    context: Arc<Context>,
    state: WorkflowState,
}

impl WorkflowExecutor {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            active_workflows: HashMap::new(),
        }
    }

    pub fn register_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.name.clone(), Arc::new(workflow));
    }

    pub async fn start_workflow(
        &mut self,
        name: &str,
        ctx: Arc<Context>,
    ) -> Result<String, WorkflowError> {
        let workflow = self.workflows.get(name)
            .ok_or_else(|| WorkflowError::StepFailed(format!("Workflow not found: {}", name)))?;
        
        let instance_id = uuid::Uuid::new_v4().to_string();
        let instance = WorkflowInstance {
            workflow: Arc::clone(workflow),
            context: ctx,
            state: WorkflowState::Created,
        };
        
        self.active_workflows.insert(instance_id.clone(), instance);
        
        // 启动工作流
        if let Some(instance) = self.active_workflows.get_mut(&instance_id) {
            instance.workflow.start(&instance.context).await?;
            instance.state = WorkflowState::Running;
        }
        
        Ok(instance_id)
    }

    pub async fn pause_workflow(&mut self, instance_id: &str) -> Result<(), WorkflowError> {
        if let Some(instance) = self.active_workflows.get_mut(instance_id) {
            instance.workflow.pause().await?;
            instance.state = WorkflowState::Paused;
            Ok(())
        } else {
            Err(WorkflowError::InvalidState)
        }
    }

    pub async fn resume_workflow(&mut self, instance_id: &str) -> Result<(), WorkflowError> {
        if let Some(instance) = self.active_workflows.get_mut(instance_id) {
            instance.workflow.resume(&instance.context).await?;
            instance.state = WorkflowState::Running;
            Ok(())
        } else {
            Err(WorkflowError::InvalidState)
        }
    }

    pub fn get_workflow_state(&self, instance_id: &str) -> Option<WorkflowState> {
        self.active_workflows.get(instance_id).map(|i| i.state.clone())
    }
} 