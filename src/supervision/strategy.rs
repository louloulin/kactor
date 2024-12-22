use async_trait::async_trait;
use std::time::Duration;
use crate::{Context, Pid, ProtoError};

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

#[derive(Clone)]
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

#[derive(Clone)]
pub struct OneForAllStrategy {
    max_retries: i32,
    within_time: Duration,
    directive: fn(&ProtoError) -> SupervisorDirective,
}

impl OneForAllStrategy {
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
impl SupervisorStrategy for OneForAllStrategy {
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
            let directive = (self.directive)(reason);
            if matches!(directive, SupervisorDirective::Restart) {
                for child_pid in ctx.children.iter() {
                    ctx.restart_child(child_pid).await;
                }
            }
            directive
        }
    }
}

#[derive(Clone)]
pub struct AllForOneStrategy {
    max_retries: i32,
    within_time: Duration,
    directive: fn(&ProtoError) -> SupervisorDirective,
}

impl AllForOneStrategy {
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
impl SupervisorStrategy for AllForOneStrategy {
    async fn handle_failure(
        &self,
        ctx: &mut Context,
        pid: &Pid,
        reason: &ProtoError,
        stats: &ChildStats,
    ) -> SupervisorDirective {
        if stats.failure_count > self.max_retries {
            for child_pid in ctx.children.iter() {
                ctx.stop_child(child_pid).await;
            }
            SupervisorDirective::Stop
        } else {
            let directive = (self.directive)(reason);
            match directive {
                SupervisorDirective::Restart => {
                    for child_pid in ctx.children.iter() {
                        ctx.restart_child(child_pid).await;
                    }
                }
                SupervisorDirective::Stop => {
                    for child_pid in ctx.children.iter() {
                        ctx.stop_child(child_pid).await;
                    }
                }
                _ => {}
            }
            directive
        }
    }
} 