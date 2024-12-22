use std::time::Duration;
use crate::dispatcher::ThreadPoolDispatcher;

#[derive(Clone)]
pub struct SystemConfig {
    pub host: String,
    pub port: u16,
    pub dispatcher: ThreadPoolDispatcher,
    pub deadletter_timeout: Duration,
    pub shutdown_timeout: Duration,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 0, // 随机端口
            dispatcher: ThreadPoolDispatcher::new(num_cpus::get()),
            deadletter_timeout: Duration::from_secs(5),
            shutdown_timeout: Duration::from_secs(10),
        }
    }
}

impl SystemConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_dispatcher(mut self, dispatcher: ThreadPoolDispatcher) -> Self {
        self.dispatcher = dispatcher;
        self
    }

    pub fn with_deadletter_timeout(mut self, timeout: Duration) -> Self {
        self.deadletter_timeout = timeout;
        self
    }

    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }
} 