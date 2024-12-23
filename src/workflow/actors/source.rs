use super::*;

#[async_trait]
pub trait SourceActor: Actor {
    // 获取数据源的元数据信息
    async fn get_metadata(&self) -> Result<SourceMetadata, WorkflowError>;
    
    // 控制数据读取的速率
    fn set_rate_limit(&mut self, records_per_second: u32);
    
    // 支持检查点和恢复
    async fn get_checkpoint(&self) -> Result<Checkpoint, WorkflowError>;
    async fn restore_from_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<(), WorkflowError>;
}

pub struct SourceMetadata {
    pub schema: Option<Schema>,
    pub estimated_size: Option<u64>,
    pub partitions: Vec<SourcePartition>,
}

#[derive(Clone)]
pub struct FileSourceActor {
    config: SourceConfig,
    format: String,
    rate_limiter: RateLimiter,
    metrics: Arc<SourceMetrics>,
    checkpoint: Option<Checkpoint>,
}

#[async_trait]
impl Actor for FileSourceActor {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        match msg {
            Message::Start => {
                self.start_reading(ctx).await?;
            }
            Message::Pause => {
                self.pause_reading().await?;
            }
            Message::Resume => {
                self.resume_reading(ctx).await?;
            }
            Message::GetMetrics => {
                self.report_metrics(ctx).await?;
            }
            _ => {}
        }
        Ok(())
    }
} 