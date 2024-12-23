use std::sync::Arc;
use tokio::runtime::Runtime;
use crate::config::SystemConfig;
use crate::dispatcher::ThreadPoolDispatcher;

pub struct ActorSystem {
    config: SystemConfig,
    runtime: Arc<Runtime>,
    dispatcher: Arc<ThreadPoolDispatcher>,
}

impl ActorSystem {
    pub fn new(config: SystemConfig) -> Self {
        let runtime = Arc::new(Runtime::new().unwrap());
        let dispatcher = Arc::new(config.dispatcher.clone());
        
        Self {
            config,
            runtime,
            dispatcher,
        }
    }

    pub fn config(&self) -> &SystemConfig {
        &self.config
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
} 