use rand::seq::SliceRandom;

pub struct GossipActor {
    cluster: Arc<Cluster>,
    gossip_size: usize,
}

impl GossipActor {
    pub fn new(cluster: Arc<Cluster>) -> Self {
        Self {
            cluster,
            gossip_size: 3, // 每次选择的gossip目标数量
        }
    }

    async fn spread_gossip(&self) {
        let members = self.cluster.get_members();
        let targets: Vec<_> = members.choose_multiple(&mut rand::thread_rng(), self.gossip_size)
            .cloned()
            .collect();

        for target in targets {
            if let Err(e) = self.send_gossip_to(&target).await {
                log::error!("Failed to send gossip to {}: {}", target.id, e);
            }
        }
    }

    async fn send_gossip_to(&self, target: &Member) -> Result<(), SendError> {
        let state = self.create_gossip_state();
        // 发送gossip状态
        // ...
        Ok(())
    }

    fn create_gossip_state(&self) -> GossipState {
        GossipState {
            members: self.cluster.get_members(),
            partitions: self.cluster.partitions.iter()
                .map(|p| p.clone())
                .collect(),
        }
    }
}

#[async_trait]
impl Actor for GossipActor {
    async fn receive(&mut self, ctx: &Context, msg: Message) -> Result<(), SendError> {
        match msg {
            Message::Gossip => {
                self.spread_gossip().await;
            }
            Message::GossipState(state) => {
                self.merge_state(state).await?;
            }
            _ => {}
        }
        Ok(())
    }
} 