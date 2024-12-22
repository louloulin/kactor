use async_trait::async_trait;
use std::time::Duration;
use tokio::task::JoinHandle;
use crate::{Context, Message, SendError};
use crate::dispatcher::strategy::{RoundRobinState, LeastBusyState};
use crate::dispatcher::worker::{WorkerPool, Worker};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::dispatcher::priority::{PriorityDispatcher, Priority};
use crate::dispatcher::backpressure::{BackpressureConfig, BackpressureController};

#[derive(Debug, Clone, Copy)]
pub enum ScheduleStrategy {
    RoundRobin,
    LeastBusy,
    Random,
}

#[derive(Debug)]
pub struct ThreadPoolMetrics {
    pub active_threads: usize,
    pub queued_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub avg_processing_time: Duration,
}

#[derive(Debug)]
pub struct LoadBalancing {
    pub max_concurrent_tasks: usize,
    pub task_queue_size: usize,
    pub worker_count: usize,
}

#[async_trait]
pub trait Dispatcher: Send + Sync {
    // 核心调度功能
    async fn dispatch(&self, ctx: &Context, msg: Message) -> Result<(), SendError>;
    async fn shutdown(&self);

    // 调度策略
    fn schedule_strategy(&self) -> ScheduleStrategy;
    
    // 负载均衡
    fn load_balancing(&self) -> &LoadBalancing;
    
    // 指标收集
    fn metrics(&self) -> ThreadPoolMetrics;
}

pub struct ThreadPoolDispatcher {
    worker_pool: Arc<WorkerPool>,
    strategy: ScheduleStrategy,
    load_balancing: LoadBalancing,
    metrics: parking_lot::RwLock<ThreadPoolMetrics>,
    round_robin_state: once_cell::sync::OnceCell<RoundRobinState>,
    least_busy_state: once_cell::sync::OnceCell<LeastBusyState>,
    priority_dispatcher: Arc<RwLock<PriorityDispatcher>>,
    backpressure: Arc<BackpressureController>,
}

impl ThreadPoolDispatcher {
    pub fn new(config: ThreadPoolConfig) -> Self {
        let worker_pool = Arc::new(WorkerPool::new(config.load_balancing.worker_count));

        let priority_dispatcher = Arc::new(RwLock::new(
            PriorityDispatcher::new(config.load_balancing.task_queue_size)
        ));

        let backpressure = Arc::new(BackpressureController::new(
            BackpressureConfig::default()
        ));

        Self {
            worker_pool,
            strategy: config.strategy,
            load_balancing: config.load_balancing,
            metrics: parking_lot::RwLock::new(ThreadPoolMetrics::default()),
            round_robin_state: once_cell::sync::OnceCell::new(),
            least_busy_state: once_cell::sync::OnceCell::new(),
            priority_dispatcher,
            backpressure,
        }
    }

    fn update_metrics(&self, start: std::time::Instant, success: bool) {
        let mut metrics = self.metrics.write();
        if success {
            metrics.completed_tasks += 1;
        } else {
            metrics.failed_tasks += 1;
        }
        
        let duration = start.elapsed();
        metrics.avg_processing_time = (metrics.avg_processing_time + duration) / 2;
    }

    async fn dispatch_round_robin(&self, ctx: Context, msg: Message) -> Result<(), SendError> {
        let state = self.round_robin_state.get_or_init(|| {
            RoundRobinState::new(self.load_balancing.worker_count)
        });

        let worker_id = state.next_worker();
        self.spawn_on_worker(worker_id, ctx, msg).await
    }

    async fn dispatch_least_busy(&self, ctx: Context, msg: Message) -> Result<(), SendError> {
        let state = self.least_busy_state.get_or_init(|| {
            LeastBusyState::new(self.load_balancing.worker_count)
        });

        let worker_id = state.get_least_busy_worker();
        state.update_load(worker_id, 1);

        let result = self.spawn_on_worker(worker_id, ctx, msg).await;
        state.update_load(worker_id, -1);

        result
    }

    async fn dispatch_random(&self, ctx: Context, msg: Message) -> Result<(), SendError> {
        let worker_id = rand::thread_rng().gen_range(0..self.load_balancing.worker_count);
        self.spawn_on_worker(worker_id, ctx, msg).await
    }

    async fn spawn_on_worker(&self, worker_id: usize, ctx: Context, msg: Message) -> Result<(), SendError> {
        // 检查队列大小限制
        if self.metrics.read().queued_tasks >= self.load_balancing.task_queue_size {
            return Err(SendError::MailboxFull);
        }

        // 检查并发任务限制
        if self.metrics.read().active_threads >= self.load_balancing.max_concurrent_tasks {
            return Err(SendError::MailboxFull);
        }

        let start = std::time::Instant::now();
        
        let worker = self.worker_pool
            .get_worker(worker_id)
            .ok_or(SendError::SystemShuttingDown)?;

        let result = worker.send_task(ctx, msg);

        Ok(())
    }

    pub async fn dispatch_with_priority(
        &self,
        priority: Priority,
        ctx: Context,
        msg: Message,
    ) -> Result<(), SendError> {
        // 检查反压
        self.backpressure.try_acquire().await?;

        let result = {
            let mut dispatcher = self.priority_dispatcher.write().await;
            dispatcher.dispatch_with_priority(priority, ctx, msg)
        };

        if result.is_err() {
            self.backpressure.release();
        }

        result
    }
}

#[async_trait]
impl Dispatcher for ThreadPoolDispatcher {
    async fn dispatch(&self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        let start = std::time::Instant::now();
        let ctx = ctx.clone();
        
        {
            let mut metrics = self.metrics.write();
            metrics.queued_tasks += 1;
            metrics.active_threads += 1;
        }

        let result = match self.strategy {
            ScheduleStrategy::RoundRobin => {
                // 实现轮询调度
                self.dispatch_round_robin(ctx, msg).await
            }
            ScheduleStrategy::LeastBusy => {
                // 实现最小负载调度
                self.dispatch_least_busy(ctx, msg).await
            }
            ScheduleStrategy::Random => {
                // 实现随机调度
                self.dispatch_random(ctx, msg).await
            }
        };

        {
            let mut metrics = self.metrics.write();
            metrics.queued_tasks -= 1;
            metrics.active_threads -= 1;
        }

        self.update_metrics(start, result.is_ok());
        result
    }

    async fn shutdown(&self) {
        self.worker_pool.shutdown().await;
    }

    fn schedule_strategy(&self) -> ScheduleStrategy {
        self.strategy
    }

    fn load_balancing(&self) -> &LoadBalancing {
        &self.load_balancing
    }

    fn metrics(&self) -> ThreadPoolMetrics {
        self.metrics.read().clone()
    }
}

// 配置结构体
pub struct ThreadPoolConfig {
    pub strategy: ScheduleStrategy,
    pub load_balancing: LoadBalancing,
    pub thread_stack_size: usize,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            strategy: ScheduleStrategy::RoundRobin,
            load_balancing: LoadBalancing {
                max_concurrent_tasks: 1000,
                task_queue_size: 10000,
                worker_count: num_cpus::get(),
            },
            thread_stack_size: 3 * 1024 * 1024, // 3MB
        }
    }
} 