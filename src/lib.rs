mod commands;
mod handlers;

use std::env;
use std::io;

use serenity::client::Client;
use serenity::prelude::SerenityError;
use serenity::framework::standard::StandardFramework;

use tracing::{
    subscriber::{set_global_default, SetGlobalDefaultError},
    Level
};
use tracing_appender::rolling::daily;
use tracing_subscriber::{
    fmt::{Layer, writer::MakeWriterExt},
    layer::SubscriberExt,
    registry,
};

use commands::GENERAL_GROUP;
use handlers::Handler;

pub struct Config {
    token: String,
    prefix: &'static str,
    is_debug: bool,
}

impl Config {
    pub fn new() -> Result<Config, env::VarError> {
        let token = env::var("DISCORD_TOKEN")?;
        let prefix = "!";
        let is_debug = env::var("IS_DEBUG").is_ok();

        Ok(Config {
            token, prefix, is_debug
        })
    }
}

pub fn logging_init(config: &Config) -> Result<(), SetGlobalDefaultError> {
    let stdout_level = match config.is_debug {
        true => Level::INFO,
        false => Level::WARN,
    };

    let file_appender = daily("./log", "bot.log");

    let subscriber = registry()
        .with(
            Layer::new()
                .with_writer(io::stdout.with_max_level(stdout_level))
        )
        .with(
            Layer::new()
                .with_writer(file_appender.with_max_level(Level::INFO))
                .json()
        );

    set_global_default(subscriber)
}

pub async fn bot_builder(config: Config)-> Result<Client, SerenityError> {
    let framework = StandardFramework::new()
        .configure(|c| {
            //set prefix
            c.prefix(config.prefix)
        })

        //add command groups
        .group(&GENERAL_GROUP)

        ;

    let client = Client::builder(&config.token)
        .event_handler(Handler)
        .framework(framework)
        .await?;

    Ok(client)
}
