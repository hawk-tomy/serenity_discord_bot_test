use serenity::async_trait;
use serenity::client::EventHandler;
use serenity::model::{gateway::Ready, interactions::Interaction};
use serenity::prelude::*;

use tracing::info;

use crate::app_cmd::{interaction_handler, setup_app_cmd};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // Log at the INFO level. This is a macro from the `tracing` crate.
        info!("{} is connected!", ready.user.name);
        setup_app_cmd(&ctx).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        interaction_handler(ctx, interaction).await;
    }
}
