use std::sync::{Arc, Mutex};
use crate::{Actor, Context, Message, SystemMessage, ChildStats};

pub(crate) struct ActorCell {
    actor: Arc<Mutex<Box<dyn Actor>>>,
    context: Context,
    state: ActorState,
    child_stats: Arc<DashMap<Pid, ChildStats>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ActorState {
    Starting,
    Running,
    Stopping,
    Stopped,
}

impl ActorCell {
    pub fn new(actor: Box<dyn Actor>, context: Context) -> Self {
        Self {
            actor: Arc::new(Mutex::new(actor)),
            context,
            state: ActorState::Starting,
            child_stats: Arc::new(DashMap::new()),
        }
    }

    pub async fn handle_message(&mut self, msg: Message) {
        if self.state == ActorState::Running {
            let mut actor = self.actor.lock().unwrap();
            actor.receive(&mut self.context, msg).await;
        }
    }

    pub async fn handle_system_message(&mut self, msg: SystemMessage) {
        match msg {
            SystemMessage::Stop => self.stop().await,
            SystemMessage::Restart => self.restart().await,
            SystemMessage::Resume => self.resume().await,
            SystemMessage::Suspend => self.suspend().await,
            SystemMessage::Failure(error) => self.handle_failure(error).await,
            SystemMessage::Watch(pid) => self.watch(pid),
            SystemMessage::Unwatch(pid) => self.unwatch(&pid),
            SystemMessage::Terminated(pid) => self.handle_terminated(pid).await,
        }
    }

    async fn stop(&mut self) {
        if self.state != ActorState::Stopped {
            self.state = ActorState::Stopping;
            let mut actor = self.actor.lock().unwrap();
            actor.stopped(&mut self.context).await;
            self.state = ActorState::Stopped;
        }
    }

    async fn restart(&mut self) {
        self.stop().await;
        self.state = ActorState::Starting;
        let mut actor = self.actor.lock().unwrap();
        actor.started(&mut self.context).await;
        self.state = ActorState::Running;
    }

    fn resume(&mut self) {
        self.state = ActorState::Running;
    }

    fn suspend(&mut self) {
        self.state = ActorState::Stopping;
    }

    fn watch(&mut self, pid: Pid) {
        self.context.watchers.insert(pid);
    }

    fn unwatch(&mut self, pid: &Pid) {
        self.context.watchers.remove(pid);
    }

    async fn handle_terminated(&mut self, pid: Pid) {
        // Notify watchers
        for watcher in &self.context.watchers {
            self.context.send(watcher, Message {
                payload: Box::new(SystemMessage::Terminated(pid.clone())),
                sender: Some(self.context.self_pid.clone()),
                header: None,
            }).await.ok();
        }
    }

    async fn handle_failure(&mut self, error: SendError) {
        if let Some(ref strategy) = self.context.supervisor_strategy {
            let stats = self.child_stats
                .entry(self.context.self_pid.clone())
                .or_insert_with(ChildStats::default);
            
            stats.record_failure();
            
            let directive = strategy.handle_failure(
                &mut self.context,
                &self.context.self_pid,
                &ProtoError::SendError(error),
                &stats,
            ).await;

            match directive {
                SupervisorDirective::Resume => self.resume(),
                SupervisorDirective::Restart => self.restart().await,
                SupervisorDirective::Stop => self.stop().await,
                SupervisorDirective::Escalate => {
                    // 将错误上报给父 actor
                    if let Some(ref parent) = self.context.parent {
                        self.context.send(parent, Message {
                            payload: Box::new(SystemMessage::Failure(error)),
                            sender: Some(self.context.self_pid.clone()),
                            header: None,
                        }).await.ok();
                    }
                }
            }
        }
    }
} 