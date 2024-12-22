use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::cluster::{Member, Partition, ClusterConfig};

#[derive(Debug)]
pub struct ClusterState {
    config: Arc<ClusterConfig>,
    members: Arc<DashMap<String, Member>>,
    partitions: Arc<DashMap<String, Partition>>,
    local_member: Arc<RwLock<Member>>,
    metrics: Arc<ClusterMetrics>,
}

impl ClusterState {
    pub fn new(config: ClusterConfig) -> Self {
        let metrics = Arc::new(ClusterMetrics::new(&config.name));
        
        Self {
            config: Arc::new(config),
            members: Arc::new(DashMap::new()),
            partitions: Arc::new(DashMap::new()),
            local_member: Arc::new(RwLock::new(Member::default())),
            metrics,
        }
    }

    pub async fn update_member(&self, member: Member) {
        let old_status = self.members.get(&member.id)
            .map(|m| m.status.clone());
            
        self.members.insert(member.id.clone(), member.clone());
        
        // 更新指标
        let (alive, suspect, dead) = self.count_member_states();
        self.metrics.update_member_counts(alive, suspect, dead);
        
        // 如果状态发生变化，可能需要重新平衡
        if old_status.map(|s| s != member.status).unwrap_or(true) {
            self.check_rebalance_needed().await;
        }
    }

    pub async fn remove_member(&self, member_id: &str) {
        if self.members.remove(member_id).is_some() {
            let (alive, suspect, dead) = self.count_member_states();
            self.metrics.update_member_counts(alive, suspect, dead);
            self.check_rebalance_needed().await;
        }
    }

    pub async fn update_partition(&self, partition: Partition) {
        self.partitions.insert(partition.id.clone(), partition);
        self.update_partition_metrics();
    }

    fn count_member_states(&self) -> (usize, usize, usize) {
        let mut alive = 0;
        let mut suspect = 0;
        let mut dead = 0;

        for member in self.members.iter() {
            match member.status {
                MemberStatus::Alive => alive += 1,
                MemberStatus::Suspect => suspect += 1,
                MemberStatus::Dead => dead += 1,
            }
        }

        (alive, suspect, dead)
    }

    async fn check_rebalance_needed(&self) {
        let score = self.calculate_balance_score();
        self.metrics.update_partition_balance(score);
        
        if score > self.config.rebalance_threshold {
            // 触发再平衡
            // ...
        }
    }

    fn calculate_balance_score(&self) -> f64 {
        // 计算分区分布的不平衡程度
        // 返回0.0-1.0之间的值，0表示完全平衡
        // ...
        0.0
    }

    fn update_partition_metrics(&self) {
        self.metrics.partition_count.set(self.partitions.len() as f64);
        
        // 计算其他分区相关指标
        // ...
    }
} 