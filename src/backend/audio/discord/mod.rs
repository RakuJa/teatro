use serenity::futures::io;
use songbird::error::JoinError;
use std::time::Duration;

mod discord_bot;
pub mod discord_handle;
mod event_handler;
mod pcm_source;

#[derive(Debug, thiserror::Error)]
pub enum DiscordError {
    #[error("failed to build the Discord client: {0}")]
    Client(#[source] Box<serenity::Error>),
    #[error("Discord gateway did not become ready within {0:?}")]
    ReadyTimeout(Duration),
    #[error("failed to join the voice channel: {0}")]
    Join(#[from] JoinError),
    #[error("failed to start the async runtime: {0}")]
    Runtime(#[source] io::Error),
    #[error("Discord gateway failed: {0}")]
    Gateway(#[source] Box<serenity::Error>),
}
