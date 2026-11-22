use crate::audio::output::engine::Engine;
use crate::audio::output::{OUT_CHANNELS, OUT_RATE, OutputKind};
use crate::backend::audio::discord::DiscordError;
use crate::backend::audio::discord::event_handler::EventHandler;
use crate::backend::audio::discord::pcm_source::PcmSource;
use serenity::Client;
use serenity::all::{ChannelId, GatewayIntents, GuildId};
use songbird::input::{Input, RawAdapter};
use songbird::tracks::TrackHandle;
use songbird::{SerenityInit, Songbird};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, oneshot};
use tokio::task::JoinHandle;
use tracing::warn;

const READY_TIMEOUT: Duration = Duration::from_secs(20);

pub struct DiscordBot {
    manager: Arc<Songbird>,
    engine: Engine,
    guild: GuildId,
    channel: ChannelId,
    track: Option<TrackHandle>,
    gateway: JoinHandle<()>,
    shard_manager: Arc<serenity::gateway::ShardManager>,
}

impl DiscordBot {
    /// Logs in and waits until the gateway is ready. Does not join a channel yet.
    /// Logs in and waits until the gateway is ready. Does not join a channel yet.
    pub async fn connect(
        token: &str,
        guild_id: u64,
        channel_id: u64,
        engine: Engine,
    ) -> Result<Self, DiscordError> {
        let manager = Songbird::serenity();
        let ready = Arc::new(Notify::new());

        let mut client = Client::builder(
            token,
            GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES,
        )
        .event_handler(EventHandler {
            ready: Arc::clone(&ready),
        })
        .register_songbird_with(Arc::clone(&manager))
        .await
        .map_err(|e| DiscordError::Client(Box::new(e)))?;

        let shard_manager = Arc::clone(&client.shard_manager);

        // Reports a gateway failure (bad token, disallowed intents, ...) right
        // away instead of letting `connect` wait out the whole ready timeout.
        let (err_tx, err_rx) = oneshot::channel();
        let gateway = tokio::spawn(async move {
            if let Err(e) = client.start().await {
                warn!("Discord gateway stopped: {e}");
                let _ = err_tx.send(e);
            }
        });

        tokio::select! {
            () = ready.notified() => {}
            Ok(e) = err_rx => {
                gateway.abort();
                return Err(DiscordError::Gateway(Box::new(e)));
            }
            () = tokio::time::sleep(READY_TIMEOUT) => {
                gateway.abort();
                return Err(DiscordError::ReadyTimeout(READY_TIMEOUT));
            }
        }

        Ok(Self {
            manager,
            engine,
            guild: GuildId::new(guild_id),
            channel: ChannelId::new(channel_id),
            track: None,
            gateway,
            shard_manager,
        })
    }

    async fn is_streaming(&self) -> bool {
        let Some(track) = &self.track else {
            return false;
        };
        if track.get_info().await.is_err() {
            return false; // track ended or was stopped
        }
        match self.manager.get(self.guild) {
            Some(call) => call.lock().await.current_channel().is_some(),
            None => false,
        }
    }

    /// Joins the channel, starts the track, and only then confirms Discord as
    /// the active output. If joining fails, local playback is untouched.
    ///
    /// The takeover only happens if Discord is still the *wanted* output, so a
    /// quick switch back to local while this was joining is respected.
    pub async fn start_streaming(&mut self) -> Result<(), DiscordError> {
        if self.is_streaming().await {
            self.engine.selector().confirm(OutputKind::Discord);
            return Ok(());
        }
        self.track = None; // drop any dead handle before rejoining
        let call = self.manager.join(self.guild, self.channel).await?;

        let adapter = RawAdapter::new(
            PcmSource::new(self.engine.tap(OutputKind::Discord)),
            OUT_RATE,
            OUT_CHANNELS,
        );
        self.track = Some(call.lock().await.play_input(Input::from(adapter)));

        self.engine.selector().confirm(OutputKind::Discord);
        Ok(())
    }

    /// Stops the track and leaves the channel. The selector has already been
    /// pointed at another output by the time this runs, so the tap goes quiet
    /// on its own.
    pub async fn stop_streaming(&mut self) {
        let Some(track) = self.track.take() else {
            return;
        };
        if let Err(e) = track.stop() {
            warn!("Failed to stop Discord track: {e}");
        }
        if let Err(e) = self.manager.leave(self.guild).await {
            warn!("Failed to leave voice channel: {e}");
        }
    }

    /// Stops streaming and closes the gateway connection.
    pub async fn shutdown(mut self) {
        self.stop_streaming().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        self.shard_manager.shutdown_all().await;
        if tokio::time::timeout(Duration::from_secs(2), &mut self.gateway)
            .await
            .is_err()
        {
            self.gateway.abort();
        }
    }
}
