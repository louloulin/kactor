use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::RwLock;
use crate::cluster::{ClusterState, ClusterError, ClusterEvent, ClusterEventBus};

pub struct PartitionManager {
    state: Arc<ClusterState>,
    events: Arc<ClusterEventBus>,
    partitions: Arc<DashMap<String, PartitionActor>>,
    rebalancer: Arc<RwLock<Rebalancer>>,
}

impl PartitionManager {
    pub fn new(state: Arc<ClusterState>, events: Arc<ClusterEventBus>) -> Self {
        Self {
            state,
            events,
            partitions: Arc::new(DashMap::new()),
            rebalancer: Arc::new(RwLock::new(Rebalancer::new())),
        }
    }

    pub async fn start(&self) -> Result<(), ClusterError> {
        // 初始化分区
        self.initialize_partitions().await?;
        
        // 启动再平衡器
        self.start_rebalancer().await?;

        Ok(())
    }

    async fn initialize_partitions(&self) -> Result<(), ClusterError> {
        let config = self.state.config();
        
        for i in 0..config.partition_count {
            let partition_id = format!("partition-{}", i);
            let partition = Partition {
                id: partition_id.clone(),
                owner: self.state.local_member().id.clone(),
                replicas: vec![],
                actors: Default::default(),
            };

            let actor = PartitionActor::new(
                self.state.clone(),
                partition,
            );

            self.partitions.insert(partition_id, actor);
        }

        Ok(())
    }

    pub async fn transfer_partitions(&self) -> Result<(), ClusterError> {
        let mut transfers = Vec::new();

        // 收集需要转移的分区
        for partition in self.partitions.iter() {
            if partition.owner == self.state.local_member().id {
                if let Some(new_owner) = self.find_new_owner(&partition).await {
                    transfers.push((partition.id.clone(), new_owner));
                }
            }
        }

        // 执行转移
        for (partition_id, new_owner) in transfers {
            self.transfer_partition(&partition_id, &new_owner).await?;
        }

        Ok(())
    }

    async fn transfer_partition(
        &self,
        partition_id: &str,
        new_owner: &str,
    ) -> Result<(), ClusterError> {
        let partition = self.partitions.get(partition_id)
            .ok_or_else(|| ClusterError::Partition(
                format!("Partition not found: {}", partition_id)
            ))?;

        // 执行转移
        partition.transfer_to(new_owner).await?;

        // 发布事件
        self.events.publish(ClusterEvent::PartitionMoved {
            partition_id: partition_id.to_string(),
            from: self.state.local_member().id.clone(),
            to: new_owner.to_string(),
            timestamp: std::time::SystemTime::now(),
        });

        Ok(())
    }

    async fn find_new_owner(&self, partition: &Partition) -> Option<String> {
        // 实现寻找新所有者的逻辑
        // 可以基于负载、位置等因素
        None
    }
} 