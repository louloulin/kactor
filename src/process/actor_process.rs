use async_trait::async_trait;
use tokio::sync::mpsc;
use std::sync::Arc;
use crate::{Actor, Context, Message, Pid, Props, Process, SendError};

pub struct ActorProcess<A: Actor> {
    context: Context,
    sender: mpsc::UnboundedSender<Message>,
    _phantom: std::marker::PhantomData<A>,
}

impl<A: Actor> ActorProcess<A> {
    pub fn new(context: Context, props: Props<A>) -> Self {
        let sender = context.sender.clone();
        
        // Spawn actor message processing loop
        let mut ctx = context.clone();
        tokio::spawn(async move {
            // Start the actor
            ctx.actor.started(&mut ctx).await;

            // Process messages
            while let Some(msg) = ctx.mailbox.recv().await {
                let next = Next {
                    middleware: &ctx.middleware,
                    actor: &mut ctx.actor,
                };
                
                if let Err(err) = next.run(&mut ctx, msg).await {
                    // Handle error through supervision
                    ctx.handle_failure(err).await;
                }
            }

            // Stop the actor
            ctx.actor.stopped(&mut ctx).await;
        });
        
        Self {
            context,
            sender,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<A: Actor> Process for ActorProcess<A> {
    async fn send_message(&self, message: Message) -> Result<(), SendError> {
        self.sender.send(message)
            .map_err(|_| SendError::MailboxFull)
    }

    fn pid(&self) -> Pid {
        self.context.self_pid.clone()
    }
}

impl<A: Actor> Clone for ActorProcess<A> {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            sender: self.sender.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
} 