pub struct Props<A: Actor> {
    actor_producer: Box<dyn Fn() -> A + Send + Sync>,
    middleware: Vec<Box<dyn Middleware>>,
    supervisor_strategy: Option<Box<dyn SupervisorStrategy>>,
}

impl<A: Actor> Props<A> {
    pub fn new<F>(producer: F) -> Self 
    where
        F: Fn() -> A + Send + Sync + 'static
    {
        Self {
            actor_producer: Box::new(producer),
            middleware: Vec::new(),
            supervisor_strategy: None,
        }
    }

    pub fn with_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware.push(Box::new(middleware));
        self
    }
} 