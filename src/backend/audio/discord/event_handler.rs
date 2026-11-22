use serenity::all::EventHandler as SerenityEventHandler;
use serenity::all::{Context, Ready};
use serenity::async_trait;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;

pub struct EventHandler {
    pub(crate) ready: Arc<Notify>,
}

#[async_trait]
impl SerenityEventHandler for EventHandler {
    async fn ready(&self, _: Context, data: Ready) {
        info!("Discord bot connected as {}", data.user.name);
        self.ready.notify_one();
    }
}
