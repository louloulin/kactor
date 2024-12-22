use std::time::Duration;
use tokio::time::sleep;
use crate::{RemoteEndpoint, RemoteError};

pub struct ReconnectionStrategy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl ReconnectionStrategy {
    pub fn new(max_attempts: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay,
        }
    }

    pub async fn attempt_reconnect(&self, endpoint: &RemoteEndpoint) -> Result<(), RemoteError> {
        let mut attempts = 0;
        let mut delay = self.base_delay;

        while attempts < self.max_attempts {
            match endpoint.connect().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    log::warn!(
                        "Reconnection attempt {} failed for {}: {:?}",
                        attempts + 1,
                        endpoint.address,
                        e
                    );
                    attempts += 1;
                    sleep(delay).await;
                    delay = std::cmp::min(delay * 2, self.max_delay);
                }
            }
        }

        Err(RemoteError::ConnectionError(format!(
            "Failed to reconnect after {} attempts",
            self.max_attempts
        )))
    }
} 