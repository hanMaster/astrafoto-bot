use log::info;
use crate::stuff::error::Result;
use crate::stuff::message_handler::MessageHandler;
use crate::stuff::transport::Transport;

pub struct Poller<'a, T, H>
where
    T: Transport + 'a,
    H: MessageHandler,
{
    transport: &'a T,
    handler: H,
}
impl<'a, T, H> Poller<'a, T, H>
where
    T: Transport,
    H: MessageHandler,
{
    pub fn new(transport: &'a T, handler: H) -> Poller<'a, T, H> {
        Self { transport, handler }
    }

    pub async fn start_polling(&mut self) -> Result<()> {
        info!("Start polling...");
        loop {
            let msg = self.transport.receive_message().await?;
            self.handler.handle(msg).await?;
            self.handler.handle_awaits().await?;
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
