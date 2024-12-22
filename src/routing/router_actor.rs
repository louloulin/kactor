use async_trait::async_trait;
use crate::{Actor, Context, Message, Pid, Props, SendError};
use super::{Router, RouterKind};
use crate::routing::metrics::RouterMetrics;

pub struct RouterActor {
    router: Router,
    metrics: RouterMetrics,
}

impl RouterActor {
    pub fn new(kind: RouterKind) -> Self {
        let router = Router::new(kind);
        let metrics = RouterMetrics::new(&format!("router_{:?}", kind));
        Self { router, metrics }
    }

    pub fn with_routees(kind: RouterKind, routees: Vec<Pid>) -> Self {
        let mut actor = Self::new(kind);
        for routee in routees {
            actor.router.add_routee(routee);
        }
        actor.metrics.update_routee_count(actor.router.routee_count());
        actor
    }
}

#[async_trait]
impl Actor for RouterActor {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        let start = std::time::Instant::now();
        
        let result = match msg {
            Message::AddRoutee(pid) => {
                self.router.add_routee(pid);
                self.metrics.update_routee_count(self.router.routee_count());
                Ok(())
            }
            Message::RemoveRoutee(pid) => {
                self.router.remove_routee(&pid);
                self.metrics.update_routee_count(self.router.routee_count());
                Ok(())
            }
            Message::GetRoutees => {
                ctx.respond(self.router.get_routees());
                Ok(())
            }
            _ => {
                self.metrics.record_message();
                let result = self.router.route(&msg).await;
                if result.is_err() {
                    self.metrics.record_error();
                }
                result
            }
        };

        self.metrics.record_routing_time(start.elapsed());
        result
    }
}

// Router Props 构建器
pub struct RouterProps {
    kind: RouterKind,
    routees: Vec<Pid>,
}

impl RouterProps {
    pub fn new(kind: RouterKind) -> Self {
        Self {
            kind,
            routees: Vec::new(),
        }
    }

    pub fn with_routees(mut self, routees: Vec<Pid>) -> Self {
        self.routees = routees;
        self
    }

    pub fn spawn(self, ctx: &Context) -> Result<Pid, SendError> {
        let props = Props::new(move || {
            Box::new(RouterActor::with_routees(self.kind, self.routees.clone()))
        });
        ctx.spawn(props)
    }
} 