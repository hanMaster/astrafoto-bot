pub use crate::error::Result;
use crate::stuff::message_handler::Handler;
use crate::stuff::poller::Poller;
use crate::stuff::repository::OrderRepository;
use crate::stuff::transport::WhatsApp;

mod config;
mod error;
mod stuff;

#[tokio::main]
async fn main() -> Result<()> {
    let transport = WhatsApp::new();
    pretty_env_logger::init_timed();
    let repo = OrderRepository::new();
    let handler = Handler::new(repo, transport.clone());
    Poller::new(transport, handler).start_polling().await?;
    Ok(())
}
