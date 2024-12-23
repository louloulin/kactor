use super::*;
use std::collections::HashMap;

pub struct WorkflowCoordinator {
    workflow: Arc<DagWorkflow>,
    active_steps: HashMap<String, Pid>,
    completed_steps: HashSet<String>,
    state: WorkflowState,
}

impl WorkflowCoordinator {
    pub fn new(workflow: DagWorkflow) -> Self {
        Self {
            workflow: Arc::new(workflow),
            active_steps: HashMap::new(),
            completed_steps: HashSet::new(),
            state: WorkflowState::Created,
        }
    }

    async fn handle_step_completed(&mut self, step_name: String, result: Result<(), WorkflowError>) {
        match result {
            Ok(()) => {
                self.completed_steps.insert(step_name);
                self.active_steps.remove(&step_name);
                
                // 检查并启动下一批可执行的步骤
                self.schedule_next_steps().await;
                
                // 检查工作流是否完成
                if self.is_workflow_completed() {
                    self.state = WorkflowState::Completed;
                }
            }
            Err(e) => {
                log::error!("Step {} failed: {:?}", step_name, e);
                self.state = WorkflowState::Failed;
                self.rollback_active_steps().await;
            }
        }
    }

    async fn schedule_next_steps(&mut self) {
        let ready_steps = self.workflow.get_ready_steps(&self.completed_steps);
        for step in ready_steps {
            if let Err(e) = self.start_step(step).await {
                log::error!("Failed to start step: {:?}", e);
                self.state = WorkflowState::Failed;
                return;
            }
        }
    }

    async fn rollback_active_steps(&mut self) {
        for (step_name, pid) in &self.active_steps {
            if let Err(e) = self.send_rollback(pid).await {
                log::error!("Failed to rollback step {}: {:?}", step_name, e);
            }
        }
    }
}

#[async_trait]
impl Actor for WorkflowCoordinator {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        match msg {
            Message::StartWorkflow => {
                self.state = WorkflowState::Running;
                self.schedule_next_steps().await;
            }
            Message::StepCompleted { step_name, result } => {
                self.handle_step_completed(step_name, result).await;
            }
            Message::PauseWorkflow => {
                self.state = WorkflowState::Paused;
                // 暂停所有活动步骤
            }
            Message::ResumeWorkflow => {
                self.state = WorkflowState::Running;
                self.schedule_next_steps().await;
            }
            Message::CancelWorkflow => {
                self.state = WorkflowState::Failed;
                self.rollback_active_steps().await;
            }
            _ => {}
        }
        Ok(())
    }
} 