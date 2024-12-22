use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberStatus {
    Alive,
    Suspect,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub status: MemberStatus,
    pub labels: HashMap<String, String>,
}

pub struct MembershipActor {
    cluster: Arc<Cluster>,
    failure_detector: FailureDetector,
}

impl MembershipActor {
    pub fn new(cluster: Arc<Cluster>) -> Self {
        Self {
            cluster,
            failure_detector: FailureDetector::new(
                cluster.config.heartbeat_interval,
                3, // 允许错过的心跳次数
            ),
        }
    }

    async fn handle_heartbeat(&mut self, from: Member) {
        self.failure_detector.heartbeat(&from.id);
        
        if let Some(mut member) = self.cluster.members.get_mut(&from.id) {
            member.status = MemberStatus::Alive;
        } else {
            self.cluster.members.insert(from.id.clone(), from);
        }
    }

    async fn check_members(&mut self) {
        let now = Instant::now();
        for member in self.cluster.members.iter() {
            if self.failure_detector.is_failed(&member.id, now) {
                if let Some(mut member) = self.cluster.members.get_mut(&member.id) {
                    match member.status {
                        MemberStatus::Alive => member.status = MemberStatus::Suspect,
                        MemberStatus::Suspect => member.status = MemberStatus::Dead,
                        MemberStatus::Dead => {
                            self.cluster.members.remove(&member.id);
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Actor for MembershipActor {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        match msg {
            Message::Heartbeat(from) => {
                self.handle_heartbeat(from).await;
            }
            Message::CheckMembers => {
                self.check_members().await;
            }
            _ => {}
        }
        Ok(())
    }
} 