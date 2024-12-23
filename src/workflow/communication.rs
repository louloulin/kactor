pub struct StepChannel {
    name: String,
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
    receiver: tokio::sync::mpsc::UnboundedReceiver<Message>,
}

impl StepChannel {
    pub fn new(name: String) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        Self {
            name,
            sender,
            receiver,
        }
    }

    pub async fn send(&self, msg: Message) -> Result<(), SendError> {
        self.sender.send(msg)
            .map_err(|_| SendError::MailboxFull)
    }

    pub async fn receive(&mut self) -> Option<Message> {
        self.receiver.recv().await
    }
} 