use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::mpsc;
use dashmap::DashMap;
use std::time::Duration;
use crate::{
    Actor, ActorSystem, Message, Pid, SendError, SpawnError,
    Props, SupervisorStrategy, Middleware, SystemMessage,
    ProtoError, ChildStats, MessageQueue, QueueMessage,
    ActorCell, ActorState,
};
use tokio::sync::RwLock;

pub struct Context {
    // Actor 和系统相关
    actor: Box<dyn Actor>,
    system: Arc<ActorSystem>,
    self_pid: Pid,
    parent: Option<Pid>,
    
    // 子 Actor 管理
    children: HashSet<Pid>,
    watchers: HashSet<Pid>,
    child_stats: DashMap<Pid, ChildStats>,
    child_props: DashMap<Pid, Props<Box<dyn Actor>>>,
    
    // 消息处理
    mailbox: MessageQueue,
    middleware: Vec<Box<dyn Middleware>>,
    
    // 监督策略
    supervisor_strategy: Option<Box<dyn SupervisorStrategy>>,
    
    // Actor 状态
    state: Arc<RwLock<ActorState>>,
    
    // 停止标志
    stopping: bool,
    
    // 持久化
    persistence: Option<Persistence>,
}

impl Context {
    pub(crate) fn new(
        actor: Box<dyn Actor>,
        system: Arc<ActorSystem>,
        self_pid: Pid,
        parent: Option<Pid>,
    ) -> Self {
        Self {
            actor,
            system,
            self_pid,
            parent,
            children: HashSet::new(),
            watchers: HashSet::new(),
            child_stats: DashMap::new(),
            child_props: DashMap::new(),
            mailbox: MessageQueue::new(),
            middleware: Vec::new(),
            supervisor_strategy: None,
            state: Arc::new(RwLock::new(ActorState::Starting)),
            stopping: false,
            persistence: None,
        }
    }

    // Actor 生命周期管理
    pub async fn start(&mut self) -> Result<(), SendError> {
        self.state.write().await.set_started();
        self.actor.started(self).await
    }

    pub async fn stop(&mut self) -> Result<(), SendError> {
        if self.stopping {
            return Ok(());
        }
        self.stopping = true;
        self.state.write().await.set_stopping();

        // 停止所有子 actor
        for child in &self.children {
            self.stop_child(child).await;
        }

        // 通知所有观察者
        self.notify_watchers_about_stop().await;

        self.actor.stopping(self).await?;
        self.actor.stopped(self).await?;
        self.state.write().await.set_stopped();

        // 从系统中移除
        self.system.process_registry.remove(&self.self_pid.id);

        Ok(())
    }

    pub async fn restart(&mut self, reason: &str) -> Result<(), SendError> {
        self.state.write().await.set_restarting();
        self.actor.restarting(self, reason).await?;
        
        // 重新初始化 Actor
        self.actor = self.props.producer();
        self.start().await
    }

    // 消息处理
    pub(crate) async fn process_messages(&mut self) {
        while let Some(msg) = self.mailbox.receive().await {
            match msg {
                QueueMessage::User(msg) => self.handle_message(msg).await,
                QueueMessage::System(msg) => self.handle_system_message(msg).await,
            }
            
            if self.stopping {
                break;
            }
        }
    }

    async fn handle_message(&mut self, msg: Message) {
        if self.state.read().await.is_stopping() {
            return;
        }

        let next = Next {
            middleware: &self.middleware,
            actor: &mut self.actor,
        };

        if let Err(err) = next.run(self, msg).await {
            self.handle_failure(err).await;
        }
    }

    async fn handle_system_message(&mut self, msg: SystemMessage) -> Result<(), SendError> {
        match msg {
            SystemMessage::Stop => self.stop().await,
            SystemMessage::Restart { reason } => self.restart(&reason).await,
            SystemMessage::Terminated(pid) => {
                self.children.remove(&pid);
                Ok(())
            }
            SystemMessage::Watch(pid) => {
                self.watchers.insert(pid);
                Ok(())
            }
            SystemMessage::Unwatch(pid) => {
                self.watchers.remove(&pid);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    // 子 Actor 管理
    pub async fn spawn<A: Actor>(&mut self, props: Props<A>) -> Result<Pid, SpawnError> {
        let child_pid = self.system.spawn(props, Some(self.self_pid.clone())).await?;
        self.children.insert(child_pid.clone());
        Ok(child_pid)
    }

    pub async fn stop_child(&mut self, pid: &Pid) {
        if self.children.remove(pid) {
            let stop_msg = SystemMessage::Stop;
            self.system.send_system(pid, stop_msg).await.ok();
        }
    }

    // 错误处理
    async fn handle_failure(&mut self, error: SendError) {
        if let Some(ref strategy) = self.supervisor_strategy {
            let stats = self.child_stats
                .entry(self.self_pid.clone())
                .or_insert_with(ChildStats::default);
            
            stats.record_failure();
            
            let directive = strategy.handle_failure(
                self,
                &self.self_pid,
                &ProtoError::SendError(error),
                stats,
            ).await;

            match directive {
                SupervisorDirective::Resume => self.state.write().await.set_running(),
                SupervisorDirective::Restart => self.restart(&error.to_string()).await,
                SupervisorDirective::Stop => self.stop().await,
                SupervisorDirective::Escalate => {
                    if let Some(ref parent) = self.parent {
                        let failure_msg = SystemMessage::Failure(error);
                        self.system.send_system(parent, failure_msg).await.ok();
                    }
                }
            }
        }
    }

    // 辅助方法
    async fn notify_watchers_about_stop(&self) {
        let terminated_msg = SystemMessage::Terminated(self.self_pid.clone());
        for watcher in &self.watchers {
            self.system.send_system(watcher, terminated_msg.clone()).await.ok();
        }
    }

    pub fn pid(&self) -> &Pid {
        &self.self_pid
    }

    pub fn parent(&self) -> Option<&Pid> {
        self.parent.as_ref()
    }

    pub fn children(&self) -> &HashSet<Pid> {
        &self.children
    }

    pub fn system(&self) -> &ActorSystem {
        &self.system
    }

    pub fn state(&self) -> ActorState {
        self.state.read().await.state
    }

    pub fn is_stopping(&self) -> bool {
        self.stopping
    }

    pub fn persistence(&self) -> &Persistence {
        self.persistence.as_ref().expect("Persistence not configured")
    }

    pub fn with_persistence(mut self, persistence: Persistence) -> Self {
        self.persistence = Some(persistence);
        self
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.stopping {
            tokio::spawn(self.stop());
        }
    }
}
