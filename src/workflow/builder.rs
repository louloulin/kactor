use super::*;

pub struct DagBuilder {
    dag: DagActor,
    system: ActorSystem,
}

impl DagBuilder {
    pub fn new(system: ActorSystem) -> Self {
        Self {
            dag: DagActor::new(),
            system,
        }
    }

    pub async fn add_step<A>(
        &mut self,
        id: String,
        actor: A,
        dependencies: HashSet<String>
    ) -> Result<(), WorkflowError>
    where
        A: ActorWorkflowStep + 'static,
    {
        let props = Props::new(move || Box::new(actor));
        let pid = self.system.spawn(props).await?;
        
        self.dag.add_node(id, pid, dependencies);
        Ok(())
    }

    pub fn add_dependency(&mut self, from: String, to: String) -> Result<(), WorkflowError> {
        if let Some(node) = self.dag.nodes.get_mut(&to) {
            node.dependencies.insert(from.clone());
            self.dag.update_topology(&to);
            Ok(())
        } else {
            Err(WorkflowError::NodeNotFound(to))
        }
    }

    pub async fn build(self) -> Result<Pid, SpawnError> {
        let props = Props::new(move || Box::new(self.dag));
        self.system.spawn(props).await
    }

    pub async fn build_from_definition(&mut self, def: &WorkflowDefinition) -> Result<(), WorkflowError> {
        // 创建 source actors
        for (id, source) in &def.sources {
            let actor = self.actor_factory.create_source_actor(
                &source.actor,
                &source.format,
                &source.config
            )?;
            self.add_step(id.clone(), actor, HashSet::new()).await?;
        }

        // 创建 transform actors
        for (id, transform) in &def.transforms {
            let actor = self.actor_factory.create_transform_actor(
                &transform.actor,
                &transform.config
            )?;
            let deps: HashSet<_> = transform.inputs.iter().cloned().collect();
            self.add_step(id.clone(), actor, deps).await?;
        }

        // 创建 sink actors
        for (id, sink) in &def.sinks {
            let actor = self.actor_factory.create_sink_actor(
                &sink.actor,
                &sink.format,
                &sink.config
            )?;
            let deps: HashSet<_> = sink.inputs.iter().cloned().collect();
            self.add_step(id.clone(), actor, deps).await?;
        }

        Ok(())
    }
} 