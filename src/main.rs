use crate::bot::Bot;
pub use crate::error::Result;
use crate::stuff::poller::Poller;
use crate::stuff::transport::WhatsApp;

mod bot;
mod data_types;
mod config;
mod error;
mod stuff;

#[tokio::main]
async fn main() -> Result<()> {
    // let mut bot = Bot::new();
    // bot.run().await;
    let transport = WhatsApp::new();
    Poller::new(transport).start_polling().await?;
    Ok(())
}
