use super::*;

pub struct WorkflowBuilder {
    workflow: DagWorkflow,
}

impl WorkflowBuilder {
    pub fn new(name: String) -> Self {
        Self {
            workflow: DagWorkflow::new(name),
        }
    }

    pub fn add_step<S: WorkflowStep + 'static>(&mut self, step: S) -> NodeIndex {
        self.workflow.add_step(step)
    }

    pub fn add_sub_workflow(&mut self, workflow: Workflow) -> NodeIndex {
        self.workflow.add_sub_workflow(workflow)
    }

    pub fn add_dependency(&mut self, from: NodeIndex, to: NodeIndex) -> &mut Self {
        self.workflow.add_dependency(from, to);
        self
    }

    pub fn add_dependency_by_name(&mut self, from: &str, to: &str) -> Result<&mut Self, WorkflowError> {
        self.workflow.add_dependency_by_name(from, to)?;
        Ok(self)
    }

    pub fn build(self) -> DagWorkflow {
        self.workflow
    }
} 