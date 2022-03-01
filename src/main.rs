extern crate serenity_discord_bot_test;

use serenity_discord_bot_test::{bot_builder, Config, logging_init};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    //load env
    dotenv().ok();

    // gen config
    let config = Config::new().unwrap_or_else(|err| {
        panic!("An error occured create config struct: {}", err);
    });

    // setup logging
    if let Err(why) = logging_init(&config) {
        panic!("An error occurred setup: {:?}", why);
    }

    // build bot
    let mut bot = bot_builder(config).await.unwrap_or_else(|err| {
        panic!("An error occured build the bot: {:?}", err);
    });

    // start listening for events by starting a single shard
    if let Err(why) = bot.start().await {
        panic!("An error occurred while running the bot: {:?}", why);
    }
}
