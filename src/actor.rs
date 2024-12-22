#[async_trait::async_trait]
pub trait Actor: Send + 'static {
    async fn started(&mut self, ctx: &mut Context) {}
    async fn stopped(&mut self, ctx: &mut Context) {}
    async fn receive(&mut self, ctx: &mut Context, msg: Message);
}

pub struct Context {
    actor: Box<dyn Actor>,
    system: Arc<ActorSystem>,
    self_pid: Pid,
    parent: Option<Pid>,
    children: HashSet<Pid>,
    watchers: HashSet<Pid>,
} 