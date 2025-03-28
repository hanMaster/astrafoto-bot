use crate::stuff::error::Result;
use crate::stuff::message_handler::MessageHandler;
use crate::stuff::transport::Transport;

pub struct Poller<T, H>
where
    T: Transport,
    H: MessageHandler,
{
    transport: T,
    handler: H,
}
impl<T, H> Poller<T, H>
where
    T: Transport,
    H: MessageHandler,
{
    pub fn new(transport: T, handler: H) -> Self {
        println!("Poller::new");
        Self { transport, handler }
    }

    pub async fn start_polling(&mut self) -> Result<()> {
        println!("Poller::start_polling");
        loop {
            let msg = self.transport.receive_message().await?;
            self.handler.handle(msg);
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
    async fn test_poll() {
        let transport = WhatsApp::new();
        let repo = OrderRepository::new();
        let handler = Handler::new(repo);
        let res = Poller::new(transport, handler).start_polling().await;

        if let Err(ref e) = res {
            eprintln!("{}", e);
            assert!(res.is_ok());
        }
    }
}
