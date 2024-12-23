#[async_trait]
pub trait TransformActor: Actor {
    // 支持窗口操作
    async fn process_window(&mut self, window: Window) -> Result<Vec<Message>, WorkflowError>;
    
    // 状态管理
    async fn save_state(&self) -> Result<Vec<u8>, WorkflowError>;
    async fn restore_state(&mut self, state: Vec<u8>) -> Result<(), WorkflowError>;
    
    // 支持动态更新配置
    fn update_config(&mut self, config: TransformConfig) -> Result<(), WorkflowError>;
}

pub struct JoinTransformActor {
    config: TransformConfig,
    state: HashMap<String, Vec<Message>>,
    window_manager: WindowManager,
    metrics: Arc<TransformMetrics>,
}

impl JoinTransformActor {
    async fn process_batch(&mut self, batch: Vec<Message>) -> Result<Vec<Message>, WorkflowError> {
        // 实现批处理逻辑
        let mut results = Vec::new();
        for msg in batch {
            if let Some(joined) = self.join_message(msg).await? {
                results.push(joined);
            }
        }
        Ok(results)
    }
} 