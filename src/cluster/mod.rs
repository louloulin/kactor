use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;
use crate::{Actor, Context, Message, Pid, SendError};

pub mod membership;
pub mod partition;
pub mod discovery;
pub mod gossip;

use membership::{Member, MemberStatus};
use partition::{Partition, PartitionActor};

#[derive(Debug, Clone)]
pub struct ClusterConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub seed_nodes: Vec<String>,
    pub partition_count: usize,
    pub heartbeat_interval: Duration,
    pub gossip_interval: Duration,
}

pub struct Cluster {
    config: ClusterConfig,
    members: Arc<DashMap<String, Member>>,
    partitions: Arc<DashMap<String, Partition>>,
    local_member: Arc<RwLock<Member>>,
}

impl Cluster {
    pub async fn new(config: ClusterConfig) -> Result<Self, ClusterError> {
        let local_member = Member {
            id: format!("{}:{}", config.host, config.port),
            host: config.host.clone(),
            port: config.port,
            status: MemberStatus::Alive,
            labels: Default::default(),
        };

        let cluster = Self {
            config,
            members: Arc::new(DashMap::new()),
            partitions: Arc::new(DashMap::new()),
            local_member: Arc::new(RwLock::new(local_member)),
        };

        Ok(cluster)
    }

    pub async fn start(&self, ctx: &Context) -> Result<(), ClusterError> {
        // 启动成员管理
        self.start_membership(ctx).await?;
        
        // 启动分区管理
        self.start_partitions(ctx).await?;
        
        // 启动发现服务
        self.start_discovery(ctx).await?;
        
        // 启动gossip协议
        self.start_gossip(ctx).await?;

        Ok(())
    }

    pub fn get_member(&self, id: &str) -> Option<Member> {
        self.members.get(id).map(|m| m.clone())
    }

    pub fn get_members(&self) -> Vec<Member> {
        self.members.iter().map(|m| m.clone()).collect()
    }

    pub fn get_partition(&self, key: &str) -> Option<Partition> {
        let partition_id = self.get_partition_id(key);
        self.partitions.get(&partition_id).map(|p| p.clone())
    }

    fn get_partition_id(&self, key: &str) -> String {
        // 使用一致性哈希确定分区
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        format!("partition-{}", hash % self.config.partition_count as u64)
    }
} 