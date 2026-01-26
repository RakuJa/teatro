use std::time::{Duration, Instant};

pub struct PlayerStates {
    pub(crate) last_refresh: Instant,
    pub(crate) last_local_update: Instant,
    pub(crate) refresh_interval: Duration,
    pub(crate) local_elapsed: u64,
    pub(crate) shuffle_on: bool,
    pub(crate) loop_on: bool,
    pub(crate) mute_on: bool,
    pub(crate) pause_on: bool,
    pub(crate) solo_on: bool,
    pub(crate) stop_all_on: bool,
}

impl Default for PlayerStates {
    fn default() -> Self {
        Self {
            last_refresh: Instant::now(),
            last_local_update: Instant::now(),
            refresh_interval: Duration::from_secs(20),
            local_elapsed: 0,
            shuffle_on: false,
            loop_on: false,
            mute_on: false,
            pause_on: false,
            solo_on: false,
            stop_all_on: false,
        }
    }
}
