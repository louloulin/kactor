use super::*;

#[derive(Debug)]
pub enum WorkflowMessage {
    // DAG 相关消息
    AddNode {
        id: String,
        actor_pid: Pid,
        dependencies: HashSet<String>,
    },
    RemoveNode {
        id: String,
    },
    AddDependency {
        from: String,
        to: String,
    },
    RemoveDependency {
        from: String,
        to: String,
    },
    
    // 执行控制消息
    Execute,
    Rollback,
    StepCompleted {
        step_name: String,
        result: Result<(), WorkflowError>,
    },
    
    // 监控消息
    SetMonitor {
        pid: Pid,
    },
}

pub struct WorkflowManagerActor {
    executor: WorkflowExecutor,
}

#[async_trait]
impl Actor for WorkflowManagerActor {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        if let Some(workflow_msg) = msg.downcast_ref::<WorkflowMessage>() {
            match workflow_msg {
                WorkflowMessage::Execute => {
                    let result = self.executor.execute_step().await;
                    let _ = ctx.send(Message::Execute).await;
                }
                WorkflowMessage::Rollback => {
                    let result = self.executor.rollback_step().await;
                    let _ = ctx.send(Message::Rollback).await;
                }
                WorkflowMessage::StepCompleted { step_name, result } => {
                    let _ = ctx.send(Message::StepCompleted { step_name, result }).await;
                }
                WorkflowMessage::PauseWorkflow => {
                    let result = self.executor.pause_workflow().await;
                    let _ = ctx.send(Message::PauseWorkflow).await;
                }
                WorkflowMessage::ResumeWorkflow => {
                    let result = self.executor.resume_workflow().await;
                    let _ = ctx.send(Message::ResumeWorkflow).await;
                }
                WorkflowMessage::CancelWorkflow => {
                    let result = self.executor.cancel_workflow().await;
                    let _ = ctx.send(Message::CancelWorkflow).await;
                }
            }
        }
        Ok(())
    }
} 