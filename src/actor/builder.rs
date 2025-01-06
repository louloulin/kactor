use crate::actor::Actor;
use crate::props::Props;
use crate::supervision::SupervisorStrategy;
use crate::middleware::Middleware;
use crate::dispatcher::Dispatcher;

pub struct ActorBuilder<A: Actor> {
    props: Props<A>,
    supervisor: Option<Box<dyn SupervisorStrategy>>,
    middleware: Vec<Box<dyn Middleware>>,
    mailbox_size: Option<usize>,
    dispatcher: Option<Box<dyn Dispatcher>>,
}

impl<A: Actor> ActorBuilder<A> {
    pub fn new<F>(producer: F) -> Self
    where
        F: Fn() -> A + Send + Sync + 'static,
    {
        Self {
            props: Props::new(producer),
            supervisor: None,
            middleware: Vec::new(),
            mailbox_size: None,
            dispatcher: None,
        }
    }

    pub fn with_supervisor<S: SupervisorStrategy + 'static>(mut self, supervisor: S) -> Self {
        self.supervisor = Some(Box::new(supervisor));
        self
    }

    pub fn with_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware.push(Box::new(middleware));
        self
    }

    pub fn with_mailbox_size(mut self, size: usize) -> Self {
        self.mailbox_size = Some(size);
        self
    }

    pub fn with_dispatcher<D: Dispatcher + 'static>(mut self, dispatcher: D) -> Self {
        self.dispatcher = Some(Box::new(dispatcher));
        self
    }

    pub fn build(self) -> Props<A> {
        let mut props = self.props;
        if let Some(supervisor) = self.supervisor {
            props = props.with_strategy(supervisor);
        }
        for middleware in self.middleware {
            props = props.with_middleware(middleware);
        }
        if let Some(size) = self.mailbox_size {
            props = props.with_mailbox_size(size);
        }
        if let Some(dispatcher) = self.dispatcher {
            props = props.with_dispatcher(dispatcher);
        }
        props
    }
} 