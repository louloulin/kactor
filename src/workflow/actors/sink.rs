#[async_trait]
pub trait SinkActor: Actor {
    // 批量写入支持
    async fn write_batch(&mut self, batch: Vec<Message>) -> Result<(), WorkflowError>;
    
    // 事务支持
    async fn begin_transaction(&mut self) -> Result<TransactionId, WorkflowError>;
    async fn commit_transaction(&mut self, tx_id: TransactionId) -> Result<(), WorkflowError>;
    async fn rollback_transaction(&mut self, tx_id: TransactionId) -> Result<(), WorkflowError>;
    
    // 健康检查
    async fn health_check(&self) -> Result<bool, WorkflowError>;
}

pub struct PostgresSinkActor {
    config: SinkConfig,
    batch_buffer: Vec<Message>,
    current_transaction: Option<TransactionId>,
    metrics: Arc<SinkMetrics>,
    connection_pool: Pool,
} 