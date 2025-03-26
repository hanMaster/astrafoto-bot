use crate::bot::Bot;
pub use crate::error::Result;
mod bot;
mod data_types;
mod config;
mod error;


fn main() -> Result<()> {
    let mut bot = Bot::new();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(bot.run());
    Ok(())
}
