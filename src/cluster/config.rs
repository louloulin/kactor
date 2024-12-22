use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    // 基础配置
    pub name: String,
    pub host: String,
    pub port: u16,
    pub seed_nodes: Vec<String>,
    
    // 分区配置
    pub partition_count: usize,
    pub min_replicas: usize,
    pub max_replicas: usize,
    
    // 成员管理配置
    pub heartbeat_interval: Duration,
    pub suspect_timeout: Duration,
    pub dead_timeout: Duration,
    
    // Gossip配置
    pub gossip_interval: Duration,
    pub gossip_fanout: usize,
    
    // 发现配置
    pub discovery_method: DiscoveryMethod,
    pub discovery_interval: Duration,
    
    // 负载均衡配置
    pub rebalance_interval: Duration,
    pub rebalance_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    Static(Vec<String>),
    Multicast {
        addr: String,
        port: u16,
    },
    Kubernetes {
        namespace: String,
        label_selector: String,
    },
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            name: "default-cluster".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8080,
            seed_nodes: vec![],
            partition_count: 100,
            min_replicas: 1,
            max_replicas: 3,
            heartbeat_interval: Duration::from_secs(1),
            suspect_timeout: Duration::from_secs(5),
            dead_timeout: Duration::from_secs(30),
            gossip_interval: Duration::from_secs(1),
            gossip_fanout: 3,
            discovery_method: DiscoveryMethod::Static(vec![]),
            discovery_interval: Duration::from_secs(5),
            rebalance_interval: Duration::from_secs(60),
            rebalance_threshold: 0.1,
        }
    }
} 