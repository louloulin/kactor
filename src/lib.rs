//! Proto.Actor implementation in Rust

// 基础模块
pub mod actor;
pub mod config;
pub mod context;
pub mod dispatcher;
pub mod errors;
pub mod message;
pub mod middleware;
pub mod supervisor;
pub mod system;
pub mod props;
pub mod supervision;
// 远程处理模块
pub mod remote;
pub mod mailbox;

// 工作流模块
pub mod workflow {
    mod actors;
    mod backpressure;
    mod checkpoint;
    mod definition;
    mod errors;
    mod factory;
    mod loader;
    mod metrics;
    mod optimization; 
    mod quality;
    mod rate_limiter;
    mod recovery;
    mod scheduler;
    mod state;
    mod window;

    pub use self::actors::{SourceActor, TransformActor, SinkActor};
    pub use self::backpressure::{BackpressureController, ThrottleCommand};
    pub use self::checkpoint::{Checkpoint, Schema, SourcePartition};
    pub use self::definition::{WorkflowDefinition, SourceDefinition, TransformDefinition, SinkDefinition};
    pub use self::errors::StepError;
    pub use self::factory::{ActorFactory, DefaultActorFactory};
    pub use self::loader::WorkflowLoader;
    pub use self::metrics::{WorkflowMetrics, SourceMetrics, TransformMetrics, SinkMetrics};
    pub use self::quality::{DataQualityMonitor, DataQualityRule};
    pub use self::rate_limiter::RateLimiter;
    pub use self::scheduler::WorkflowScheduler;
    pub use self::state::{StateManager, StateBackend, WorkflowStatus, NodeState};
    pub use self::window::{Window, WindowType, WindowManager};
}

// 重导出常用类型
pub use actor::{Actor, /* Context, */ Props};
pub use config::SystemConfig;
pub use errors::{ProtoError, SendError, SpawnError, WorkflowError};
pub use system::ActorSystem;

// 内部使用的模块
#[doc(hidden)]
pub(crate) mod utils {
    use std::time::Duration;

    pub(crate) trait DurationExt {
        fn mul_f64(self, rhs: f64) -> Self;
    }

    impl DurationExt for Duration {
        fn mul_f64(self, rhs: f64) -> Self {
            let secs = self.as_secs_f64() * rhs;
            Duration::from_secs_f64(secs)
        }
    }
}

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// 特征重导出
pub use async_trait::async_trait;

// Re-exports
pub use middleware::{Middleware, MiddlewareChain};
pub use supervisor::{SupervisorStrategy, OneForOneStrategy};
  