use async_trait::async_trait;
use std::time::Duration;
use crate::context::Context;
use crate::{SendError};

#[async_trait]
pub trait SupervisorStrategy: Send + Sync {
    async fn handle_failure(
        &self,
        ctx: &Context,
        reason: &str,
        failures: usize,
    ) -> SupervisorDirective;
}

#[derive(Debug, Clone, Copy)]
pub enum SupervisorDirective {
    Resume,    // 继续处理消息
    Restart,   // 重启 Actor
    Stop,      // 停止 Actor
    Escalate,  // 上报给父 Actor
}

pub struct OneForOneStrategy {
    max_retries: usize,
    within_time: Duration,
    directive: SupervisorDirective,
}

impl OneForOneStrategy {
    pub fn new(max_retries: usize, within_time: Duration, directive: SupervisorDirective) -> Self {
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
        ctx: &Context,
        reason: &str,
        failures: usize,
    ) -> SupervisorDirective {
        if failures > self.max_retries {
            SupervisorDirective::Stop
        } else {
            self.directive
        }
    }
}

pub struct AllForOneStrategy {
    max_retries: usize,
    within_time: Duration,
}

#[async_trait]
impl SupervisorStrategy for AllForOneStrategy {
    async fn handle_failure(
        &self,
        ctx: &Context,
        reason: &str,
        failures: usize,
    ) -> SupervisorDirective {
        if failures > self.max_retries {
            // 停止所有子 Actor
            SupervisorDirective::Stop
        } else {
            // 重启所有子 Actor
            SupervisorDirective::Restart
        }
    }
} 