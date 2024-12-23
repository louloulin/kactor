use super::*;
use std::path::Path;
use tokio::fs;

pub struct WorkflowLoader {
    registry: Arc<DashMap<String, WorkflowDefinition>>,
    actor_factory: Arc<dyn ActorFactory>,
}

impl WorkflowLoader {
    pub fn new(actor_factory: Arc<dyn ActorFactory>) -> Self {
        Self {
            registry: Arc::new(DashMap::new()),
            actor_factory,
        }
    }

    pub async fn load_directory<P: AsRef<Path>>(&self, path: P) -> Result<(), WorkflowError> {
        let mut dir = fs::read_dir(path).await?;
        
        while let Some(entry) = dir.next_entry().await? {
            if entry.path().extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                self.load_file(entry.path()).await?;
            }
        }
        
        Ok(())
    }

    pub async fn load_file<P: AsRef<Path>>(&self, path: P) -> Result<(), WorkflowError> {
        let content = fs::read_to_string(path).await?;
        let definition = WorkflowDefinition::from_yaml(&content)?;
        
        // 验证工作流定义
        definition.validate()?;
        
        self.registry.insert(definition.name.clone(), definition);
        Ok(())
    }

    pub async fn create_workflow(&self, name: &str) -> Result<DagActor, WorkflowError> {
        let definition = self.registry.get(name)
            .ok_or_else(|| WorkflowError::NotFound(name.to_string()))?;
            
        let mut builder = DagBuilder::new(self.actor_factory.create_system());
        
        // 创建所有步骤
        for (step_id, step_def) in &definition.steps {
            let actor = self.actor_factory.create_actor(&step_def.actor, &step_def.config)?;
            let dependencies = step_def.dependencies.iter()
                .map(|s| s.to_string())
                .collect();
                
            builder.add_step(step_id.clone(), actor, dependencies).await?;
        }
        
        Ok(builder.build_dag(definition.config.clone()))
    }
} 