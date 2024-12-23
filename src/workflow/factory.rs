use super::*;
use std::collections::HashMap;

#[async_trait]
pub trait ActorFactory: Send + Sync {
    fn create_source_actor(
        &self,
        actor_type: &str,
        format: &str,
        config: &SourceConfig
    ) -> Result<Box<dyn Actor>, WorkflowError>;

    fn create_transform_actor(
        &self,
        actor_type: &str,
        config: &TransformConfig
    ) -> Result<Box<dyn Actor>, WorkflowError>;

    fn create_sink_actor(
        &self,
        actor_type: &str,
        format: &str,
        config: &SinkConfig
    ) -> Result<Box<dyn Actor>, WorkflowError>;

    fn create_system(&self) -> ActorSystem;
}

pub struct DefaultActorFactory {
    system_config: SystemConfig,
    actor_builders: HashMap<String, Box<dyn Fn(&StepConfig) -> Box<dyn Actor> + Send + Sync>>,
}

impl DefaultActorFactory {
    pub fn new(system_config: SystemConfig) -> Self {
        Self {
            system_config,
            actor_builders: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, actor_type: &str, builder: F)
    where
        F: Fn(&StepConfig) -> Box<dyn Actor> + Send + Sync + 'static,
    {
        self.actor_builders.insert(
            actor_type.to_string(),
            Box::new(builder)
        );
    }
}

#[async_trait]
impl ActorFactory for DefaultActorFactory {
    fn create_actor(&self, actor_type: &str, config: &StepConfig) -> Result<Box<dyn Actor>, WorkflowError> {
        if let Some(builder) = self.actor_builders.get(actor_type) {
            Ok(builder(config))
        } else {
            Err(WorkflowError::UnknownActorType(actor_type.to_string()))
        }
    }

    fn create_system(&self) -> ActorSystem {
        ActorSystem::new(self.system_config.clone())
    }
} 