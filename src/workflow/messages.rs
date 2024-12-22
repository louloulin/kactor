use super::*;

#[derive(Debug)]
pub enum WorkflowMessage {
    Start {
        workflow_name: String,
        reply_to: oneshot::Sender<Result<String, WorkflowError>>,
    },
    Pause {
        instance_id: String,
        reply_to: oneshot::Sender<Result<(), WorkflowError>>,
    },
    Resume {
        instance_id: String,
        reply_to: oneshot::Sender<Result<(), WorkflowError>>,
    },
    GetState {
        instance_id: String,
        reply_to: oneshot::Sender<Option<WorkflowState>>,
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
                WorkflowMessage::Start { workflow_name, reply_to } => {
                    let result = self.executor.start_workflow(
                        workflow_name,
                        Arc::new(ctx.clone())
                    ).await;
                    let _ = reply_to.send(result);
                }
                WorkflowMessage::Pause { instance_id, reply_to } => {
                    let result = self.executor.pause_workflow(instance_id).await;
                    let _ = reply_to.send(result);
                }
                WorkflowMessage::Resume { instance_id, reply_to } => {
                    let result = self.executor.resume_workflow(instance_id).await;
                    let _ = reply_to.send(result);
                }
                WorkflowMessage::GetState { instance_id, reply_to } => {
                    let state = self.executor.get_workflow_state(instance_id);
                    let _ = reply_to.send(state);
                }
            }
        }
        Ok(())
    }
} 