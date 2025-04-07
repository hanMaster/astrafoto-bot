use crate::stuff::error::Result;
use crate::stuff::message_handler::MessageHandler;
use crate::stuff::transport::Transport;
use log::{error, info};

pub struct Poller<T, H>
where
    T: Transport + Send + Clone + 'static,
    H: MessageHandler + Send,
{
    transport: T,
    handler: H,
}
impl<T, H> Poller<T, H>
where
    T: Transport + Send + Clone + 'static,
    H: MessageHandler + Send,
{
    pub fn new(transport: T, handler: H) -> Poller<T, H> {
        Self { transport, handler }
    }

    pub async fn start_polling(&mut self) -> Result<()> {
        info!("Start polling...");
        let (tx, mut rx) = tokio::sync::mpsc::channel(200);
        let transport = self.transport.clone();
        tokio::spawn(async move {
            loop {
                let msg = transport.receive_message().await;
                match msg {
                    Ok(msg) => {
                        if tx.send(msg).await.is_err() {
                            error!("Error sending message in channel");
                        };
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                    }
                }
            }
        });

        loop {
            match rx.recv().await {
                Some(msg) => {
                    self.handler.handle(msg).await?;
                    self.handler.handle_awaits().await?;
                }
                None => {
                    info!("Channel closed, shutting down");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stuff::message_handler::Handler;
    use crate::stuff::repository::OrderRepository;
    use crate::stuff::transport::WhatsApp;

    #[tokio::test]
    #[ignore]
    async fn test_poll() {
        let transport = WhatsApp::new();
        let repo = OrderRepository::new();
        let handler = Handler::new(repo, &transport);
        let res = Poller::new(&transport, handler).start_polling().await;

        if let Err(ref e) = res {
            eprintln!("{}", e);
            assert!(res.is_ok());
        }
    }
}
