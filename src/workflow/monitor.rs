use super::*;
use std::time::{Duration, Instant};

pub struct WorkflowMonitor {
    workflow_id: String,
    start_time: Option<Instant>,
    step_times: HashMap<String, Duration>,
    metrics: Arc<WorkflowMetrics>,
}

impl WorkflowMonitor {
    pub fn new(workflow_id: String, metrics: Arc<WorkflowMetrics>) -> Self {
        Self {
            workflow_id,
            start_time: None,
            step_times: HashMap::new(),
            metrics,
        }
    }

    fn record_step_start(&mut self, step_name: &str) {
        self.metrics.step_started(self.workflow_id.clone(), step_name);
    }

    fn record_step_completion(&mut self, step_name: &str, duration: Duration) {
        self.step_times.insert(step_name.to_string(), duration);
        self.metrics.step_completed(
            self.workflow_id.clone(),
            step_name,
            duration
        );
    }

    fn record_workflow_completion(&self, status: WorkflowState) {
        if let Some(start) = self.start_time {
            let duration = start.elapsed();
            self.metrics.workflow_completed(
                self.workflow_id.clone(),
                status,
                duration
            );
        }
    }
}

#[async_trait]
impl Actor for WorkflowMonitor {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        match msg {
            Message::WorkflowStarted => {
                self.start_time = Some(Instant::now());
                self.metrics.workflow_started(self.workflow_id.clone());
            }
            Message::StepStarted { step_name } => {
                self.record_step_start(&step_name);
            }
            Message::StepCompleted { step_name, duration } => {
                self.record_step_completion(&step_name, duration);
            }
            Message::WorkflowCompleted { status } => {
                self.record_workflow_completion(status);
            }
            _ => {}
        }
        Ok(())
    }
} 