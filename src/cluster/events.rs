use serde::{Serialize, Deserialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterEvent {
    // 成员事件
    MemberJoined {
        member: Member,
        timestamp: SystemTime,
    },
    MemberLeft {
        member_id: String,
        timestamp: SystemTime,
    },
    MemberStatusChanged {
        member_id: String,
        old_status: MemberStatus,
        new_status: MemberStatus,
        timestamp: SystemTime,
    },

    // 分区事件
    PartitionMoved {
        partition_id: String,
        from: String,
        to: String,
        timestamp: SystemTime,
    },
    PartitionReplicaAdded {
        partition_id: String,
        replica: String,
        timestamp: SystemTime,
    },
    PartitionReplicaRemoved {
        partition_id: String,
        replica: String,
        timestamp: SystemTime,
    },

    // 集群状态事件
    ClusterStateChanged {
        old_state: String,
        new_state: String,
        timestamp: SystemTime,
    },
}

pub trait ClusterEventHandler: Send + Sync {
    fn handle_event(&self, event: ClusterEvent);
}

pub struct ClusterEventBus {
    handlers: Arc<DashMap<String, Box<dyn ClusterEventHandler>>>,
}

impl ClusterEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(DashMap::new()),
        }
    }

    pub fn subscribe(&self, id: String, handler: Box<dyn ClusterEventHandler>) {
        self.handlers.insert(id, handler);
    }

    pub fn unsubscribe(&self, id: &str) {
        self.handlers.remove(id);
    }

    pub fn publish(&self, event: ClusterEvent) {
        for handler in self.handlers.iter() {
            handler.handle_event(event.clone());
        }
    }
} 