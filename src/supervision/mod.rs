use async_trait::async_trait;
use std::time::Duration;
use crate::{Context, Pid, ProtoError};

// 已有的代码...

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
            // 当一个子 actor 失败时，重启所有子 actor
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
            // 如果失败次数超过限制，停止所有子 actor
            for child_pid in ctx.children.iter() {
                ctx.stop_child(child_pid).await;
            }
            SupervisorDirective::Stop
        } else {
            let directive = (self.directive)(reason);
            match directive {
                SupervisorDirective::Restart => {
                    // 重启所有子 actor，按照依赖顺序
                    for child_pid in ctx.children.iter() {
                        ctx.restart_child(child_pid).await;
                    }
                }
                SupervisorDirective::Stop => {
                    // 停止所有子 actor
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

#[derive(Default)]
pub struct ChildStats {
    pub failure_count: i32,
    pub last_failure: Option<std::time::Instant>,
    pub restart_count: i32,
}

impl ChildStats {
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(std::time::Instant::now());
    }

    pub fn record_restart(&mut self) {
        self.restart_count += 1;
    }

    pub fn reset(&mut self) {
        self.failure_count = 0;
        self.last_failure = None;
        self.restart_count = 0;
    }
} 