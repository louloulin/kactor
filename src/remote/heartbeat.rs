use std::time::Duration;
use tokio::time::interval;
use crate::{Message, Pid, SendError};

pub struct HeartbeatMessage {
    pub timestamp: u64,
}

pub struct HeartbeatManager {
    interval: Duration,
    timeout: Duration,
    last_heartbeat: dashmap::DashMap<String, std::time::Instant>,
}

impl HeartbeatManager {
    pub fn new(interval: Duration, timeout: Duration) -> Self {
        Self {
            interval,
            timeout,
            last_heartbeat: dashmap::DashMap::new(),
        }
    }

    pub async fn start(&self, endpoint: Arc<RemoteEndpoint>) {
        let mut ticker = interval(self.interval);
        let address = endpoint.address.clone();

        tokio::spawn(async move {
            loop {
                ticker.tick().await;
                
                // 发送心跳
                let heartbeat = HeartbeatMessage {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                if let Err(e) = endpoint.send_heartbeat(heartbeat).await {
                    log::error!("Failed to send heartbeat to {}: {:?}", address, e);
                    break;
                }
            }
        });

        // 检查心跳超时
        let timeout = self.timeout;
        let last_heartbeat = self.last_heartbeat.clone();
        
        tokio::spawn(async move {
            let mut check_interval = interval(timeout);
            loop {
                check_interval.tick().await;
                
                if let Some(last) = last_heartbeat.get(&address) {
                    if last.elapsed() > timeout {
                        log::warn!("Heartbeat timeout for {}", address);
                        // 处理连接断开
                        endpoint.handle_connection_lost().await;
                        break;
                    }
                }
            }
        });
    }

    pub fn record_heartbeat(&self, address: &str) {
        self.last_heartbeat.insert(address.to_string(), std::time::Instant::now());
    }
} 