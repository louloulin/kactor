use super::*;
use crate::{Actor, Context, Props, SystemConfig};
use std::collections::HashMap;

pub struct WorkflowExecutor {
    workflows: HashMap<String, Arc<Workflow>>,
    active_workflows: HashMap<String, WorkflowInstance>,
    system: ActorSystem,
}

pub struct WorkflowInstance {
    workflow: Arc<Workflow>,
    context: Arc<Context>,
    state: WorkflowState,
    coordinator_pid: Pid,
    monitor_pid: Pid,
}

impl WorkflowExecutor {
    pub fn new(config: SystemConfig) -> Self {
        Self {
            workflows: HashMap::new(),
            active_workflows: HashMap::new(),
            system: ActorSystem::new(config),
        }
    }

    pub async fn spawn_step_actor<A>(&self, props: Props<A>) -> Result<Pid, SpawnError> 
    where
        A: ActorWorkflowStep + 'static,
    {
        self.system.spawn(props).await
    }

    pub async fn add_actor_step<A>(
        &mut self, 
        workflow_name: &str,
        step_name: String,
        dependencies: Vec<String>,
        actor: A
    ) -> Result<(), WorkflowError>
    where
        A: ActorWorkflowStep + 'static,
    {
        let props = Props::new(move || Box::new(actor));
        let pid = self.spawn_step_actor(props).await
            .map_err(|e| WorkflowError::StepFailed(format!("Failed to spawn actor: {}", e)))?;

        let step = ActorStepWrapper {
            name: step_name,
            dependencies,
            actor_pid: pid,
        };

        if let Some(workflow) = self.workflows.get_mut(workflow_name) {
            Arc::get_mut(workflow)
                .unwrap()
                .add_step(step);
            Ok(())
        } else {
            Err(WorkflowError::NodeNotFound(workflow_name.to_string()))
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
        
        // 创建工作流协调器
        let coordinator = WorkflowCoordinator::new((*workflow).clone());
        let coordinator_props = Props::new(move || Box::new(coordinator));
        let coordinator_pid = self.system.spawn(coordinator_props).await?;
        
        // 创建工作流监控器
        let monitor = WorkflowMonitor::new(instance_id.clone(), Arc::new(WorkflowMetrics::new()));
        let monitor_props = Props::new(move || Box::new(monitor));
        let monitor_pid = self.system.spawn(monitor_props).await?;
        
        let instance = WorkflowInstance {
            workflow: Arc::clone(workflow),
            context: ctx,
            state: WorkflowState::Created,
            coordinator_pid,
            monitor_pid,
        };
        
        self.active_workflows.insert(instance_id.clone(), instance);
        
        // 启动工作流
        self.system.send(&coordinator_pid, Message::StartWorkflow).await?;
        
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