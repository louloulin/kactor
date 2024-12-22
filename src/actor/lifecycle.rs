use std::time::Duration;
use tokio::time::sleep;

pub struct ActorLifecycle {
    pub started_at: std::time::Instant,
    pub restart_count: u32,
    pub last_restart: Option<std::time::Instant>,
}

impl ActorLifecycle {
    pub fn new() -> Self {
        Self {
            started_at: std::time::Instant::now(),
            restart_count: 0,
            last_restart: None,
        }
    }

    pub async fn delay_restart(&mut self, delay: Duration) {
        sleep(delay).await;
        self.restart_count += 1;
        self.last_restart = Some(std::time::Instant::now());
    }
}

pub trait LifecycleAware: Actor {
    fn lifecycle(&self) -> &ActorLifecycle;
    fn lifecycle_mut(&mut self) -> &mut ActorLifecycle;
} 