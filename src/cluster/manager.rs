use std::sync::Arc;
use tokio::sync::RwLock;
use crate::cluster::{ClusterConfig, ClusterState, ClusterError, ClusterEventBus};
use crate::cluster::membership::MembershipActor;
use crate::cluster::partition::PartitionManager;
use crate::cluster::discovery::Discovery;
use crate::cluster::gossip::GossipActor;

pub struct ClusterManager {
    config: Arc<ClusterConfig>,
    state: Arc<ClusterState>,
    events: Arc<ClusterEventBus>,
    membership: Arc<RwLock<MembershipActor>>,
    partition_manager: Arc<PartitionManager>,
    discovery: Arc<Discovery>,
    gossip: Arc<RwLock<GossipActor>>,
}

impl ClusterManager {
    pub async fn new(config: ClusterConfig) -> Result<Self, ClusterError> {
        let state = Arc::new(ClusterState::new(config.clone()));
        let events = Arc::new(ClusterEventBus::new());
        
        let membership = Arc::new(RwLock::new(
            MembershipActor::new(state.clone(), events.clone())
        ));

        let partition_manager = Arc::new(
            PartitionManager::new(state.clone(), events.clone())
        );

        let discovery = Arc::new(
            Discovery::new(state.clone()).await?
        );

        let gossip = Arc::new(RwLock::new(
            GossipActor::new(state.clone(), events.clone())
        ));

        Ok(Self {
            config: Arc::new(config),
            state,
            events,
            membership,
            partition_manager,
            discovery,
            gossip,
        })
    }

    pub async fn start(&self) -> Result<(), ClusterError> {
        // 启动成员管理
        self.membership.write().await.start().await?;

        // 启动分区管理
        self.partition_manager.start().await?;

        // 启动发现服务
        self.discovery.start().await?;

        // 启动gossip服务
        self.gossip.write().await.start().await?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), ClusterError> {
        // 关闭各个组件
        self.gossip.write().await.shutdown().await?;
        self.discovery.shutdown().await?;
        self.partition_manager.shutdown().await?;
        self.membership.write().await.shutdown().await?;

        Ok(())
    }

    pub async fn join(&self, seed_nodes: Vec<String>) -> Result<(), ClusterError> {
        // 加入集群
        for seed in seed_nodes {
            if let Err(e) = self.discovery.join_node(&seed).await {
                log::warn!("Failed to join node {}: {}", seed, e);
            }
        }
        Ok(())
    }

    pub async fn leave(&self) -> Result<(), ClusterError> {
        // 优雅退出集群
        self.membership.write().await.leave().await?;
        self.partition_manager.transfer_partitions().await?;
        Ok(())
    }
} 