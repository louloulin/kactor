use std::collections::{HashMap, HashSet};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use super::*;

/// DAG 节点类型
#[derive(Debug)]
pub enum DagNode {
    /// 单个步骤
    Step(Box<dyn WorkflowStep>),
    /// 子工作流
    SubWorkflow(Box<Workflow>),
}

/// DAG 工作流
pub struct DagWorkflow {
    name: String,
    graph: DiGraph<DagNode, ()>,
    node_map: HashMap<String, NodeIndex>,
    state: Arc<RwLock<WorkflowState>>,
    executed_nodes: Arc<RwLock<HashSet<NodeIndex>>>,
}

impl DagWorkflow {
    pub fn new(name: String) -> Self {
        Self {
            name,
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            state: Arc::new(RwLock::new(WorkflowState::Created)),
            executed_nodes: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// 添加步骤节点
    pub fn add_step<S: WorkflowStep + 'static>(&mut self, step: S) -> NodeIndex {
        let node = DagNode::Step(Box::new(step));
        let idx = self.graph.add_node(node);
        self.node_map.insert(step.name().to_string(), idx);
        idx
    }

    /// 添加子工作流节点
    pub fn add_sub_workflow(&mut self, workflow: Workflow) -> NodeIndex {
        let name = workflow.name.clone();
        let node = DagNode::SubWorkflow(Box::new(workflow));
        let idx = self.graph.add_node(node);
        self.node_map.insert(name, idx);
        idx
    }

    /// 添加依赖关系
    pub fn add_dependency(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, ());
    }

    /// 添加依赖关系（通过名称）
    pub fn add_dependency_by_name(&mut self, from: &str, to: &str) -> Result<(), WorkflowError> {
        let from_idx = self.node_map.get(from)
            .ok_or_else(|| WorkflowError::NodeNotFound(from.to_string()))?;
        let to_idx = self.node_map.get(to)
            .ok_or_else(|| WorkflowError::NodeNotFound(to.to_string()))?;
        
        self.graph.add_edge(*from_idx, *to_idx, ());
        Ok(())
    }

    /// 获取所有根节点（没有入边的节点）
    fn get_root_nodes(&self) -> Vec<NodeIndex> {
        self.graph.node_indices()
            .filter(|&idx| self.graph.neighbors_directed(idx, Direction::Incoming).count() == 0)
            .collect()
    }

    /// 获取节点的所有依赖
    fn get_dependencies(&self, node: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors_directed(node, Direction::Incoming).collect()
    }

    /// 检查节点是否可以执行
    async fn can_execute(&self, node: NodeIndex) -> bool {
        let executed = self.executed_nodes.read().await;
        let deps = self.get_dependencies(node);
        deps.iter().all(|&dep| executed.contains(&dep))
    }

    /// 执行节点
    async fn execute_node(&self, node: NodeIndex, ctx: &Context) -> Result<(), WorkflowError> {
        let node_data = &self.graph[node];
        match node_data {
            DagNode::Step(step) => {
                step.execute(ctx).await?;
            }
            DagNode::SubWorkflow(workflow) => {
                workflow.start(ctx).await?;
            }
        }
        
        self.executed_nodes.write().await.insert(node);
        Ok(())
    }

    /// 执行工作流
    pub async fn execute(&self, ctx: &Context) -> Result<(), WorkflowError> {
        *self.state.write().await = WorkflowState::Running;
        
        // 获取所有根节点
        let mut ready_nodes = self.get_root_nodes();
        let mut completed_nodes = HashSet::new();
        
        // 循环执行直到所有节点都完成
        while !ready_nodes.is_empty() {
            let mut next_ready_nodes = Vec::new();
            
            // 并行执行所有就绪的节点
            let executions = ready_nodes.iter().map(|&node| {
                self.execute_node(node, ctx)
            });
            
            futures::future::try_join_all(executions).await?;
            
            // 将执行完的节点添加到已完成集合
            completed_nodes.extend(ready_nodes.iter());
            
            // 查找下一批可以执行的节点
            for node in self.graph.node_indices() {
                if !completed_nodes.contains(&node) && self.can_execute(node).await {
                    next_ready_nodes.push(node);
                }
            }
            
            ready_nodes = next_ready_nodes;
        }
        
        *self.state.write().await = WorkflowState::Completed;
        Ok(())
    }
} 