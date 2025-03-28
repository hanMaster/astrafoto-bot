use crate::bot::Bot;
pub use crate::error::Result;
mod bot;
mod data_types;
mod config;
mod error;
mod stuff;

#[tokio::main]
async fn main() -> Result<()> {
    let mut bot = Bot::new();
    bot.run().await;
    Ok(())
}
