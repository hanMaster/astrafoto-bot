mod bot;
mod data_types;
use crate::bot::Bot;

fn main() {
    let mut bot = Bot::from_env();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(bot.run())
}
