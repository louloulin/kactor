use async_trait::async_trait;
use crate::actor::Actor;
use crate::workflow::definition::{SourceDefinition, TransformDefinition, SinkDefinition};
use crate::errors::WorkflowError;

#[async_trait]
pub trait ActorFactory: Send + Sync {
    async fn create_source(&self, def: &SourceDefinition) -> Result<Box<dyn Actor>, WorkflowError>;
    async fn create_transform(&self, def: &TransformDefinition) -> Result<Box<dyn Actor>, WorkflowError>;
    async fn create_sink(&self, def: &SinkDefinition) -> Result<Box<dyn Actor>, WorkflowError>;
}

pub struct DefaultActorFactory;

#[async_trait]
impl ActorFactory for DefaultActorFactory {
    async fn create_source(&self, def: &SourceDefinition) -> Result<Box<dyn Actor>, WorkflowError> {
        todo!("Implement source actor creation")
    }

    async fn create_transform(&self, def: &TransformDefinition) -> Result<Box<dyn Actor>, WorkflowError> {
        todo!("Implement transform actor creation")
    }

    async fn create_sink(&self, def: &SinkDefinition) -> Result<Box<dyn Actor>, WorkflowError> {
        todo!("Implement sink actor creation")
    }
} 