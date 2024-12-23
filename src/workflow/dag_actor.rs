use super::*;
use std::collections::{HashMap, HashSet};

pub struct DagActorNode {
    id: String,
    actor_pid: Pid,
    dependencies: HashSet<String>,
    dependents: HashSet<String>,
    state: NodeState,
    retry_count: u32,
    max_retries: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    Pending,
    Running,
    Completed,
    Failed,
}

pub struct DagActor {
    nodes: HashMap<String, DagActorNode>,
    topology: HashMap<String, Vec<String>>, // node_id -> dependent_ids
    monitor_pid: Option<Pid>,
    state: WorkflowState,
    config: WorkflowConfig,
    context: WorkflowContext,
    active_transactions: HashMap<String, WorkflowTransaction>,
}

impl DagActor {
    pub fn new(config: WorkflowConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            topology: HashMap::new(),
            monitor_pid: None,
            state: WorkflowState::Created,
            config,
            context: WorkflowContext::new(uuid::Uuid::new_v4().to_string()),
            active_transactions: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: String, actor_pid: Pid, dependencies: HashSet<String>) {
        let node = DagActorNode {
            id: id.clone(),
            actor_pid,
            dependencies,
            dependents: HashSet::new(),
            state: NodeState::Pending,
            retry_count: 0,
            max_retries: 3,
        };
        
        self.nodes.insert(id.clone(), node);
        self.update_topology(&id);
    }

    fn update_topology(&mut self, node_id: &str) {
        if let Some(node) = self.nodes.get(node_id) {
            for dep_id in &node.dependencies {
                if let Some(dep_node) = self.nodes.get_mut(dep_id) {
                    dep_node.dependents.insert(node_id.to_string());
                }
                self.topology
                    .entry(dep_id.clone())
                    .or_default()
                    .push(node_id.to_string());
            }
        }
    }

    async fn start_ready_nodes(&mut self, ctx: &Context) -> Result<(), WorkflowError> {
        let ready_nodes: Vec<String> = self.nodes
            .iter()
            .filter(|(_, node)| {
                node.state == NodeState::Pending && 
                node.dependencies.iter().all(|dep_id| {
                    self.nodes.get(dep_id)
                        .map(|n| n.state == NodeState::Completed)
                        .unwrap_or(false)
                })
            })
            .map(|(id, _)| id.clone())
            .collect();

        for node_id in ready_nodes {
            self.start_node(&node_id, ctx).await?;
        }
        Ok(())
    }

    async fn start_node(&mut self, node_id: &str, ctx: &Context) -> Result<(), WorkflowError> {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.state = NodeState::Running;
            
            // 通知监控器
            if let Some(monitor) = &self.monitor_pid {
                ctx.send(monitor, Message::StepStarted {
                    step_name: node_id.to_string()
                }).await?;
            }

            // 发送执行消息给 actor
            ctx.send(&node.actor_pid, Message::Execute).await?;
        }
        Ok(())
    }

    async fn handle_node_completed(
        &mut self,
        node_id: String,
        result: Result<(), WorkflowError>,
        ctx: &Context
    ) -> Result<(), WorkflowError> {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            match result {
                Ok(()) => {
                    node.state = NodeState::Completed;
                    // 启动依赖此节点的其他节点
                    self.start_ready_nodes(ctx).await?;
                }
                Err(e) => {
                    if node.retry_count < node.max_retries {
                        node.retry_count += 1;
                        node.state = NodeState::Pending;
                        self.start_node(&node_id, ctx).await?;
                    } else {
                        node.state = NodeState::Failed;
                        self.handle_node_failure(&node_id, ctx).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_node_failure(&mut self, node_id: &str, ctx: &Context) -> Result<(), WorkflowError> {
        self.state = WorkflowState::Failed;
        
        // 回滚所有运行中的节点
        for (id, node) in &self.nodes {
            if node.state == NodeState::Running {
                ctx.send(&node.actor_pid, Message::Rollback).await?;
            }
        }
        Ok(())
    }

    fn is_completed(&self) -> bool {
        self.nodes.values().all(|node| node.state == NodeState::Completed)
    }

    async fn start_transaction(&mut self, steps: Vec<String>) -> Result<String, WorkflowError> {
        let tx_id = uuid::Uuid::new_v4().to_string();
        let mut transaction = WorkflowTransaction::new();
        
        for step_id in steps {
            if let Some(node) = self.nodes.get(&step_id) {
                // 添加步骤和对应的补偿操作
                transaction.add_step(
                    StepWrapper::new(node.clone()),
                    CompensationWrapper::new(node.clone())
                );
            }
        }
        
        self.active_transactions.insert(tx_id.clone(), transaction);
        Ok(tx_id)
    }

    async fn commit_transaction(&mut self, tx_id: &str, ctx: &Context) -> Result<(), WorkflowError> {
        if let Some(tx) = self.active_transactions.get_mut(tx_id) {
            tx.execute(ctx).await?;
            self.active_transactions.remove(tx_id);
        }
        Ok(())
    }

    async fn rollback_transaction(&mut self, tx_id: &str, ctx: &Context) -> Result<(), WorkflowError> {
        if let Some(tx) = self.active_transactions.get_mut(tx_id) {
            tx.rollback(ctx).await?;
            self.active_transactions.remove(tx_id);
        }
        Ok(())
    }
}

#[async_trait]
impl Actor for DagActor {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        match msg {
            Message::StartWorkflow => {
                self.state = WorkflowState::Running;
                self.start_ready_nodes(ctx).await?;
            }
            Message::StepCompleted { step_name, result } => {
                self.handle_node_completed(step_name, result, ctx).await?;
                
                if self.is_completed() {
                    self.state = WorkflowState::Completed;
                    if let Some(monitor) = &self.monitor_pid {
                        ctx.send(monitor, Message::WorkflowCompleted {
                            status: self.state.clone()
                        }).await?;
                    }
                }
            }
            Message::SetMonitor { pid } => {
                self.monitor_pid = Some(pid);
            }
            _ => {}
        }
        Ok(())
    }
} 