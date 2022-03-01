use serenity::client::Context;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult,
};
use serenity::model::channel::Message;
use tracing::info;

#[group]
#[commands(ping)]
pub struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "pong!").await?;

    info!("ping command is used.");

    Ok(())
}
