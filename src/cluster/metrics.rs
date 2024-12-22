use metrics::{Counter, Gauge, Histogram};
use std::sync::Arc;
use dashmap::DashMap;

pub struct ClusterMetrics {
    // 成员指标
    member_count: Gauge,
    alive_members: Gauge,
    suspect_members: Gauge,
    dead_members: Gauge,
    
    // 分区指标
    partition_count: Gauge,
    partition_moves: Counter,
    partition_balance_score: Gauge,
    
    // 网络指标
    message_count: Counter,
    message_size: Histogram,
    gossip_round_time: Histogram,
    
    // 节点指标
    node_loads: Arc<DashMap<String, Gauge>>,
}

impl ClusterMetrics {
    pub fn new(cluster_name: &str) -> Self {
        Self {
            member_count: Gauge::new(&format!("cluster_{}_member_count", cluster_name)),
            alive_members: Gauge::new(&format!("cluster_{}_alive_members", cluster_name)),
            suspect_members: Gauge::new(&format!("cluster_{}_suspect_members", cluster_name)),
            dead_members: Gauge::new(&format!("cluster_{}_dead_members", cluster_name)),
            
            partition_count: Gauge::new(&format!("cluster_{}_partition_count", cluster_name)),
            partition_moves: Counter::new(&format!("cluster_{}_partition_moves", cluster_name)),
            partition_balance_score: Gauge::new(&format!("cluster_{}_partition_balance", cluster_name)),
            
            message_count: Counter::new(&format!("cluster_{}_message_count", cluster_name)),
            message_size: Histogram::new(&format!("cluster_{}_message_size", cluster_name)),
            gossip_round_time: Histogram::new(&format!("cluster_{}_gossip_round_time", cluster_name)),
            
            node_loads: Arc::new(DashMap::new()),
        }
    }

    pub fn update_member_counts(&self, alive: usize, suspect: usize, dead: usize) {
        self.member_count.set((alive + suspect + dead) as f64);
        self.alive_members.set(alive as f64);
        self.suspect_members.set(suspect as f64);
        self.dead_members.set(dead as f64);
    }

    pub fn record_partition_move(&self) {
        self.partition_moves.increment(1);
    }

    pub fn update_partition_balance(&self, score: f64) {
        self.partition_balance_score.set(score);
    }

    pub fn record_message(&self, size: usize) {
        self.message_count.increment(1);
        self.message_size.record(size as f64);
    }

    pub fn record_gossip_round(&self, duration: std::time::Duration) {
        self.gossip_round_time.record(duration.as_secs_f64());
    }

    pub fn update_node_load(&self, node_id: &str, load: f64) {
        self.node_loads
            .entry(node_id.to_string())
            .or_insert_with(|| Gauge::new(&format!("node_{}_load", node_id)))
            .set(load);
    }
} 