use std::sync::Arc;
use dashmap::DashMap;
use crate::SendError;

pub struct DispatcherRegistry {
    dispatchers: DashMap<String, Arc<dyn Dispatcher>>,
}

impl DispatcherRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            dispatchers: DashMap::new(),
        };

        // 注册默认调度器
        registry.register_default_dispatchers();
        registry
    }

    fn register_default_dispatchers(&mut self) {
        // 默认线程池调度器
        let default_config = ThreadPoolConfig::default();
        let default_dispatcher = Arc::new(ThreadPoolDispatcher::new(default_config));
        self.dispatchers.insert("default".to_string(), default_dispatcher);

        // 单线程调度器
        let single_thread_config = ThreadPoolConfig {
            load_balancing: LoadBalancing {
                worker_count: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        let single_thread_dispatcher = Arc::new(ThreadPoolDispatcher::new(single_thread_config));
        self.dispatchers.insert("single-thread".to_string(), single_thread_dispatcher);
    }

    pub fn register(&self, name: &str, dispatcher: Arc<dyn Dispatcher>) {
        self.dispatchers.insert(name.to_string(), dispatcher);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Dispatcher>> {
        self.dispatchers.get(name).map(|d| d.clone())
    }

    pub fn remove(&self, name: &str) {
        self.dispatchers.remove(name);
    }
} 