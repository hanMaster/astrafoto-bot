use crate::config::config;
use crate::stuff::error::Result;
use crate::stuff::message_handler::MessageHandler;
use crate::stuff::route::get_router;
use log::info;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct Poller<H: MessageHandler> {
    handler: H,
}

impl<H> Poller<H>
where
    H: MessageHandler,
{
    pub fn new(handler: H) -> Poller<H> {
        Self { handler }
    }

    pub async fn start_polling(&mut self) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let port = config().HOOK_PORT;
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            let listener = TcpListener::bind(addr).await.expect("Failed to bind");

            let app = get_router(tx.clone());

            info!("Hook server started on port {}", port);
            axum::serve(listener, app).await.expect("Axum server error");
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
        let handler = Handler::new(repo, transport);
        let res = Poller::new(handler).start_polling().await;

        if let Err(ref e) = res {
            eprintln!("{}", e);
            assert!(res.is_ok());
        }
    }
}
