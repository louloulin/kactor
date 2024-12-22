#[macro_export]
macro_rules! persistent_actor {
    ($actor:ident, $state:ty, $event:ty) => {
        #[async_trait::async_trait]
        impl PersistentActor for $actor {
            type State = $state;
            type Event = $event;

            fn persistence_id(&self) -> String {
                format!("{}-{}", stringify!($actor), self.id)
            }

            async fn recover(&mut self, ctx: &mut Context, recovery: Recovery) -> Result<(), Box<dyn Error>> {
                // 恢复快照
                if let Some(snapshot) = recovery.snapshot {
                    let state: Self::State = bincode::deserialize(&snapshot.state)?;
                    self.state = state;
                }

                // 重放事件
                for event in recovery.events {
                    let evt: Self::Event = bincode::deserialize(&event.payload)?;
                    self.update_state(&evt);
                }

                Ok(())
            }

            async fn persist_event(&mut self, ctx: &mut Context, event: Event) -> Result<(), Box<dyn Error>> {
                ctx.persistence()
                    .persist_event(self, ctx, event)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn Error>)
            }

            async fn persist_snapshot(&mut self, ctx: &mut Context, state: Self::State) -> Result<(), Box<dyn Error>> {
                ctx.persistence()
                    .create_snapshot(self, ctx)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn Error>)
            }
        }
    };
} 