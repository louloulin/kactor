#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    // 基本配置
    pub strategy: ScheduleStrategy,
    pub worker_count: usize,
    
    // 负载均衡配置
    pub max_concurrent_tasks: usize,
    pub task_queue_size: usize,
    
    // 轮询调度配置
    pub round_robin_batch_size: usize,
    
    // 最小负载配置
    pub load_update_interval: Duration,
    pub load_threshold: usize,
    
    // 随机调度配置
    pub random_seed: Option<u64>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            strategy: ScheduleStrategy::RoundRobin,
            worker_count: num_cpus::get(),
            max_concurrent_tasks: 1000,
            task_queue_size: 10000,
            round_robin_batch_size: 10,
            load_update_interval: Duration::from_millis(100),
            load_threshold: 100,
            random_seed: None,
        }
    }
} 