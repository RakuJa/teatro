use crate::audio::output::engine::Engine;
use crate::audio::output::{OutputBackend, OutputKind};
use crate::backend::audio::discord::DiscordError;
use crate::backend::audio::discord::discord_bot::DiscordBot;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::warn;

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

enum Command {
    Select,
    Release,
}

pub struct DiscordHandle {
    rt: Option<Runtime>,
    tx: Option<mpsc::UnboundedSender<Command>>,
    worker: Option<JoinHandle<()>>,
}

impl DiscordHandle {
    pub fn connect(
        token: &str,
        guild_id: u64,
        channel_id: u64,
        engine: &Engine,
    ) -> Result<Self, DiscordError> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(DiscordError::Runtime)?;

        let mut bot = rt.block_on(DiscordBot::connect(
            token,
            guild_id,
            channel_id,
            engine.clone(),
        ))?;

        let (tx, mut rx) = mpsc::unbounded_channel();
        let worker = rt.spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    Command::Select => {
                        if let Err(e) = bot.start_streaming().await {
                            warn!("Could not start Discord streaming: {e}");
                        }
                    }
                    Command::Release => bot.stop_streaming().await,
                }
            }
            // All senders dropped: the handle is being dropped.
            bot.shutdown().await;
        });

        Ok(Self {
            rt: Some(rt),
            tx: Some(tx),
            worker: Some(worker),
        })
    }

    fn send(&self, cmd: Command) {
        if let Some(tx) = &self.tx {
            // Only fails if the worker is gone, in which case there is nothing to do.
            let _ = tx.send(cmd);
        }
    }
}

impl OutputBackend for DiscordHandle {
    fn kind(&self) -> OutputKind {
        OutputKind::Discord
    }

    fn select(&self) {
        self.send(Command::Select);
    }

    fn release(&self) {
        self.send(Command::Release);
    }
}

impl Drop for DiscordHandle {
    fn drop(&mut self) {
        // Closing the channel ends the worker loop, which then shuts the bot down.
        drop(self.tx.take());
        if let (Some(rt), Some(worker)) = (self.rt.take(), self.worker.take()) {
            rt.block_on(async {
                let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, worker).await;
            });
            rt.shutdown_timeout(Duration::from_secs(1));
        }
    }
}
