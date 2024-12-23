use super::*;

pub struct DataProcessingStep {
    data: Vec<String>,
}

#[async_trait]
impl Actor for DataProcessingStep {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        match msg {
            Message::Execute => {
                self.execute_step(ctx).await?;
            }
            Message::Rollback => {
                self.rollback_step(ctx).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl ActorWorkflowStep for DataProcessingStep {
    async fn execute_step(&mut self, ctx: &Context) -> Result<(), WorkflowError> {
        // 执行数据处理逻辑
        for item in &mut self.data {
            *item = item.to_uppercase();
        }
        Ok(())
    }

    async fn rollback_step(&mut self, ctx: &Context) -> Result<(), WorkflowError> {
        // 回滚数据处理
        self.data.clear();
        Ok(())
    }
} 