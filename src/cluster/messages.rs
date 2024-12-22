use serde::{Serialize, Deserialize};
use crate::cluster::membership::Member;
use crate::cluster::partition::Partition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterMessage {
    // 成员管理消息
    Join {
        member: Member,
    },
    Leave {
        member_id: String,
    },
    Heartbeat {
        from: Member,
        incarnation: u64,
    },

    // 分区管理消息
    PartitionOwnershipRequest {
        partition_id: String,
        member_id: String,
    },
    PartitionOwnershipResponse {
        partition_id: String,
        owner: String,
        replicas: Vec<String>,
    },
    PartitionRebalance {
        partitions: Vec<Partition>,
    },

    // Gossip 消息
    GossipState {
        members: Vec<Member>,
        partitions: Vec<Partition>,
        incarnation: u64,
    },
    GossipSync {
        from: Member,
        to: Member,
        state: Box<GossipState>,
    },

    // 发现消息
    DiscoveryPing {
        from: Member,
    },
    DiscoveryPong {
        from: Member,
        known_members: Vec<Member>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipState {
    pub members: Vec<Member>,
    pub partitions: Vec<Partition>,
    pub incarnation: u64,
} 