use serenity::async_trait;
use serenity::client::{EventHandler, Context};
use serenity::model::prelude::Ready;
use tracing::info;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        // Log at the INFO level. This is a macro from the `tracing` crate.
        info!("{} is connected!", ready.user.name);
    }
}
