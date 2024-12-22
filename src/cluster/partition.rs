use std::collections::HashMap;
use consistent_hash_ring::ConsistentHashRing;
use protoactor::prelude::*;
use protoactor::cluster::messages::ClusterMessage;

#[derive(Debug, Clone)]
pub struct Partition {
    pub id: String,
    pub owner: String,
    pub replicas: Vec<String>,
    pub actors: HashMap<String, Pid>,
}

pub struct PartitionActor {
    cluster: Arc<Cluster>,
    partition: Partition,
    ring: ConsistentHashRing<String>,
}

impl PartitionActor {
    pub fn new(cluster: Arc<Cluster>, partition: Partition) -> Self {
        let mut ring = ConsistentHashRing::new();
        ring.add_node(partition.owner.clone());
        for replica in &partition.replicas {
            ring.add_node(replica.clone());
        }

        Self {
            cluster,
            partition,
            ring,
        }
    }

    async fn handle_actor_placement(&mut self, actor_id: String) -> Result<Pid, SendError> {
        let node = self.ring.get_node(&actor_id)
            .ok_or(SendError::NoRoutee)?;
            
        if node == self.partition.owner {
            // 在本地创建 actor
            let props = Props::new(move || {
                // 创建 actor 实例
                Box::new(YourActor::new())
            });
            
            let pid = ctx.spawn(props)?;
            self.partition.actors.insert(actor_id, pid.clone());
            Ok(pid)
        } else {
            // 转发到其他节点
            let member = self.cluster.get_member(&node)
                .ok_or(SendError::NoRoutee)?;
                
            let remote_pid = RemotePid::new(member.host, member.port, actor_id);
            Ok(remote_pid)
        }
    }

    async fn handle_rebalance(&mut self) {
        // 计算理想分布
        let members = self.cluster.get_members();
        let total_weight: u64 = members.iter()
            .map(|m| m.weight())
            .sum();

        // 计算每个节点应该拥有的分区数量
        let mut target_distribution = HashMap::new();
        for member in &members {
            let target = (member.weight() as f64 / total_weight as f64 
                * self.cluster.config.partition_count as f64).round() as usize;
            target_distribution.insert(member.id.clone(), target);
        }

        // 计算需要移动的分区
        let mut moves = Vec::new();
        for (partition_id, partition) in self.cluster.partitions.iter() {
            let current_owner = partition.owner.clone();
            let current_count = self.cluster.get_member_partition_count(&current_owner);
            
            if let Some(&target_count) = target_distribution.get(&current_owner) {
                if current_count > target_count {
                    // 寻找负载最小的节点
                    if let Some(new_owner) = self.find_least_loaded_member(&target_distribution) {
                        moves.push((partition_id, current_owner, new_owner));
                    }
                }
            }
        }

        // 执行分区迁移
        for (partition_id, from, to) in moves {
            self.migrate_partition(partition_id, from, to).await?;
        }
    }

    async fn migrate_partition(
        &mut self,
        partition_id: String,
        from: String,
        to: String,
    ) -> Result<(), SendError> {
        // 1. 通知源节点准备迁移
        let prepare_msg = ClusterMessage::PrepareMigration {
            partition_id: partition_id.clone(),
            to: to.clone(),
        };
        self.send_to_member(&from, prepare_msg).await?;

        // 2. 复制数据
        let copy_msg = ClusterMessage::CopyPartitionData {
            partition_id: partition_id.clone(),
            from: from.clone(),
        };
        self.send_to_member(&to, copy_msg).await?;

        // 3. 切换所有权
        let switch_msg = ClusterMessage::SwitchPartitionOwnership {
            partition_id,
            from,
            to,
        };
        self.broadcast_to_cluster(switch_msg).await?;

        Ok(())
    }
}

#[async_trait]
impl Actor for PartitionActor {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        match msg {
            Message::PlaceActor { id } => {
                self.handle_actor_placement(id).await?;
            }
            Message::Rebalance => {
                self.handle_rebalance().await;
            }
            _ => {}
        }
        Ok(())
    }
} 