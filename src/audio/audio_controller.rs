use crate::audio::output::engine::Engine;
use crate::audio::output::{Output, OutputKind};
use crate::backend::audio::discord::discord_handle::DiscordHandle;
use crate::states::settings_data::SettingsData;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::Duration;
use tracing::{info, warn};

const CONNECT_DEBOUNCE: Duration = Duration::from_millis(800);

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiscordCreds {
    token: String,
    guild: u64,
    channel: u64,
}

impl DiscordCreds {
    fn from_settings(s: &SettingsData) -> Option<Self> {
        let token = s.discord_bot_token.trim();
        if token.is_empty() {
            return None;
        }
        let id = |v: &str| v.trim().parse::<u64>().ok().filter(|&n| n != 0);
        Some(Self {
            token: token.to_owned(),
            guild: id(&s.discord_server_id)?,
            channel: id(&s.discord_voice_channel_id)?,
        })
    }
}

#[derive(Debug)]
struct State {
    /// Credentials of the Discord connection we want
    creds: Option<DiscordCreds>,
    desired: OutputKind,
    /// Bumped whenever `creds` changes; lets a slow connect detect it is stale.
    generation: u64,
}

pub struct AudioController {
    engine: Engine,
    output: Output,
    state: Mutex<State>,
}

impl std::fmt::Debug for AudioController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioController")
            .field("active", &self.output.active())
            .finish_non_exhaustive()
    }
}

static CONTROLLER: OnceLock<Arc<AudioController>> = OnceLock::new();
impl AudioController {
    pub fn install(engine: Engine, output: Output) -> Arc<Self> {
        let controller = Self::new(engine, output);
        if CONTROLLER.set(Arc::clone(&controller)).is_err() {
            warn!("AudioController was already installed");
        }
        controller
    }

    pub fn global() -> Option<&'static Arc<Self>> {
        CONTROLLER.get()
    }

    pub fn new(engine: Engine, output: Output) -> Arc<Self> {
        Arc::new(Self {
            engine,
            output,
            state: Mutex::new(State {
                creds: None,
                desired: OutputKind::Local,
                generation: 0,
            }),
        })
    }

    pub fn apply(self: &Arc<Self>, s: &SettingsData) {
        info!("Applying audio settings: output={:?}", s.audio_output);

        let creds = (s.audio_output == OutputKind::Discord)
            .then(|| DiscordCreds::from_settings(s))
            .flatten();

        let reconnect = {
            let mut st = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            st.desired = s.audio_output;
            if st.creds == creds {
                None
            } else {
                st.creds.clone_from(&creds);
                st.generation += 1;
                Some(st.generation)
            }
        };

        if let Some(generation) = reconnect {
            if let Some(old) = self.output.take(OutputKind::Discord) {
                std::thread::spawn(move || drop(old));
            }
            if let Some(c) = creds {
                self.spawn_connect(c, generation);
            }
        }
        self.output.use_output(s.audio_output);
    }

    fn is_current(&self, generation: u64) -> bool {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .generation
            == generation
    }

    /// Lets a later save retry after a failed or panicked connect.
    fn reset_creds(&self, generation: u64) {
        let mut st = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if st.generation == generation {
            st.creds = None;
        }
    }

    fn spawn_connect(self: &Arc<Self>, c: DiscordCreds, generation: u64) {
        let this = Arc::clone(self);
        std::thread::spawn(move || {
            std::thread::sleep(CONNECT_DEBOUNCE);
            if !this.is_current(generation) {
                return;
            }

            let result = catch_unwind(AssertUnwindSafe(|| {
                DiscordHandle::connect(&c.token, c.guild, c.channel, &this.engine)
            }));

            match result {
                Ok(Ok(handle)) => {
                    let st = this.state.lock().unwrap_or_else(PoisonError::into_inner);
                    if st.generation == generation {
                        this.output.add(handle);
                        this.output.use_output(st.desired);
                        drop(st);
                        info!("Discord output ready");
                    } else {
                        // Stale connection: release the lock before tearing it down.
                        drop(st);
                        drop(handle);
                    }
                }
                Ok(Err(e)) => {
                    warn!("Discord output unavailable: {e}");
                    this.reset_creds(generation);
                }
                Err(_) => {
                    warn!("Discord connect thread panicked");
                    this.reset_creds(generation);
                }
            }
        });
    }
}
