pub use crate::error::Result;
use crate::stuff::logger::Logger;
use crate::stuff::message_handler::Handler;
use crate::stuff::poller::Poller;
use crate::stuff::repository::OrderRepository;
use crate::stuff::transport::WhatsApp;

mod bot;
mod config;
mod error;
mod stuff;

#[tokio::main]
async fn main() -> Result<()> {
    let transport = WhatsApp::new();
    log::set_logger(&Logger::new(&transport)).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let repo = OrderRepository::new();
    let handler = Handler::new(repo, &transport);
    Poller::new(&transport, handler).start_polling().await?;
    Ok(())
}
