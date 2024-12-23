use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    
    pub sources: HashMap<String, SourceDefinition>,
    pub transforms: HashMap<String, TransformDefinition>,
    pub sinks: HashMap<String, SinkDefinition>,
    
    #[serde(default)]
    pub config: WorkflowConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceDefinition {
    pub actor: String,
    pub format: String,
    pub config: SourceConfig,
    pub retry: Option<RetryPolicy>,
    pub timeout: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransformDefinition {
    pub actor: String,
    pub inputs: Vec<String>,
    pub config: TransformConfig,
    pub retry: Option<RetryPolicy>,
    pub timeout: Option<String>,
    pub parallelism: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SinkDefinition {
    pub actor: String,
    pub inputs: Vec<String>,
    pub format: String,
    pub config: SinkConfig,
    pub retry: Option<RetryPolicy>,
    pub timeout: Option<String>,
    pub compensation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SourceConfig {
    pub location: String,
    pub params: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TransformConfig {
    pub operation: String,
    pub params: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SinkConfig {
    pub location: String,
    pub params: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub delay: String,
    pub multiplier: f32,
}

impl WorkflowDefinition {
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    pub fn validate(&self) -> Result<(), WorkflowError> {
        // 验证步骤依赖关系是否形成环
        self.check_cycles()?;
        
        // 验证所有依赖的步骤是否存在
        self.check_dependencies()?;
        
        // 验证超时时间格式
        self.validate_timeouts()?;
        
        Ok(())
    }
} 