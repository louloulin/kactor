use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::RwLock;
use crate::{
    ProcessRegistry, Dispatcher, Remote, Metrics,
    Message, Pid, SendError, SpawnError, Props,
    SystemConfig, Actor,
};

pub struct ActorSystem {
    // 核心组件
    pub(crate) process_registry: Arc<ProcessRegistry>,
    pub(crate) dispatcher: Arc<dyn Dispatcher>,
    pub(crate) metrics: Arc<Metrics>,
    
    // 可选组件
    pub(crate) remote: Option<Arc<Remote>>,
    
    // 系统状态
    pub(crate) config: SystemConfig,
    pub(crate) shutting_down: RwLock<bool>,
    
    // 扩展存储
    extensions: DashMap<String, Box<dyn Any + Send + Sync>>,
}

impl ActorSystem {
    pub fn new(config: SystemConfig) -> Self {
        let process_registry = Arc::new(ProcessRegistry::new());
        let dispatcher = Arc::new(config.dispatcher.clone());
        let metrics = Arc::new(Metrics::new());

        Self {
            process_registry,
            dispatcher,
            metrics,
            remote: None,
            config,
            shutting_down: RwLock::new(false),
            extensions: DashMap::new(),
        }
    }

    // Actor 生命周期管理
    pub async fn spawn<A: Actor>(&self, props: Props<A>, parent: Option<Pid>) -> Result<Pid, SpawnError> {
        if *self.shutting_down.read().await {
            return Err(SpawnError::SystemShuttingDown);
        }

        let pid = self.process_registry.next_pid();
        let context = Context::new(
            props.create_actor(),
            Arc::new(self.clone()),
            pid.clone(),
            parent,
        );

        let process = Process::new(context, props);
        self.process_registry.add(pid.clone(), process).await?;

        // 启动 actor
        if let Some(process) = self.process_registry.get(&pid.id) {
            process.start().await;
        }

        Ok(pid)
    }

    pub async fn stop(&self, pid: &Pid) {
        if let Some(process) = self.process_registry.get(&pid.id) {
            process.stop().await;
        }
    }

    // 消息发送
    pub async fn send(&self, target: &Pid, message: Message) -> Result<(), SendError> {
        // 检查是否是远程 actor
        if target.address != self.config.host {
            if let Some(remote) = &self.remote {
                return remote.send(target, message).await;
            }
            return Err(SendError::NoRemoteAvailable);
        }

        // 本地发送
        if let Some(process) = self.process_registry.get(&target.id) {
            process.send_user_message(message).await
        } else {
            Err(SendError::DeadLetter)
        }
    }

    pub async fn send_system(&self, target: &Pid, message: SystemMessage) -> Result<(), SendError> {
        if let Some(process) = self.process_registry.get(&target.id) {
            process.send_system_message(message).await
        } else {
            Err(SendError::DeadLetter)
        }
    }

    // 系统管理
    pub async fn shutdown(&self) {
        *self.shutting_down.write().await = true;

        // 停止所有 actor
        for pid in self.process_registry.get_all() {
            self.stop(&pid).await;
        }

        // 关闭远程系统
        if let Some(remote) = &self.remote {
            remote.shutdown().await;
        }

        // 关闭调度器
        self.dispatcher.shutdown().await;
    }

    // 扩展管理
    pub fn set_extension<T: Any + Send + Sync>(&self, extension: T) {
        self.extensions.insert(
            std::any::type_name::<T>().to_string(),
            Box::new(extension),
        );
    }

    pub fn get_extension<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.extensions
            .get(&std::any::type_name::<T>().to_string())
            .and_then(|ext| ext.downcast_ref())
    }

    // 远程功能
    pub fn with_remote(mut self, remote_config: RemoteConfig) -> Self {
        self.remote = Some(Arc::new(Remote::new(
            Arc::new(self.clone()),
            remote_config,
        )));
        self
    }

    // 指标收集
    pub async fn record_message_sent(&self, actor_type: &str) {
        self.metrics.record_message_sent(actor_type).await;
    }

    pub async fn record_message_received(&self, actor_type: &str) {
        self.metrics.record_message_received(actor_type).await;
    }
}

impl Clone for ActorSystem {
    fn clone(&self) -> Self {
        Self {
            process_registry: Arc::clone(&self.process_registry),
            dispatcher: Arc::clone(&self.dispatcher),
            metrics: Arc::clone(&self.metrics),
            remote: self.remote.clone(),
            config: self.config.clone(),
            shutting_down: RwLock::new(*self.shutting_down.read().blocking_lock()),
            extensions: self.extensions.clone(),
        }
    }
} 