use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait SupervisorStrategy: Send + Sync {
    async fn handle_failure(
        &self,
        ctx: &mut Context,
        pid: &Pid,
        reason: &ProtoError,
        stats: &ChildStats,
    ) -> SupervisorDirective;
}

#[derive(Debug, Clone, Copy)]
pub enum SupervisorDirective {
    Resume,
    Restart,
    Stop,
    Escalate,
}

pub struct OneForOneStrategy {
    max_retries: i32,
    within_time: Duration,
    directive: fn(&ProtoError) -> SupervisorDirective,
}

impl OneForOneStrategy {
    pub fn new(
        max_retries: i32,
        within_time: Duration,
        directive: fn(&ProtoError) -> SupervisorDirective,
    ) -> Self {
        Self {
            max_retries,
            within_time,
            directive,
        }
    }
}

#[async_trait]
impl SupervisorStrategy for OneForOneStrategy {
    async fn handle_failure(
        &self,
        ctx: &mut Context,
        pid: &Pid,
        reason: &ProtoError,
        stats: &ChildStats,
    ) -> SupervisorDirective {
        if stats.failure_count > self.max_retries {
            SupervisorDirective::Stop
        } else {
            (self.directive)(reason)
        }
    }
} 