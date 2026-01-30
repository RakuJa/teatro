use crate::states::button_states::ToggleStates;
use bitflags::bitflags;
use std::time::{Duration, Instant};

pub struct PlayerInfo {
    pub(crate) last_refresh: Instant,
    pub(crate) last_local_update: Instant,
    pub(crate) refresh_interval: Duration,
    pub(crate) local_elapsed: u64,
    pub(crate) status: PlayerStatus,
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct PlayerStatus: u16 {
        const PAUSE_MUSIC = 1 << 0;
        const SOLO_MUSIC  = 1 << 1;
        const MUTE_ALL    = 1 << 2;
        //const REC_ARM     = 1 << 3;
        const LOOP        = 1 << 4;
        const STOP_ALL    = 1 << 5;
        //const VOLUME      = 1 << 6;
        //const PAN         = 1 << 7;
        const SHUFFLE     = 1 << 8;
        //const SEND        = 1 << 8;
        //const DEVICE      = 1 << 9;
        const SHIFT       = 1 << 10;
        //const FILTER      = 1 << 11;
        //const START       = 1 << 12;
    }
}

impl PlayerStatus {
    pub const fn is_everything_stopped(self) -> bool {
        self.contains(Self::STOP_ALL)
    }
    pub const fn is_music_playable(self) -> bool {
        self.contains(Self::STOP_ALL) || self.contains(Self::PAUSE_MUSIC)
    }

    pub const fn is_shuffle_requested(self) -> bool {
        self.contains(Self::SHUFFLE)
    }

    pub const fn is_music_muted(self) -> bool {
        self.contains(Self::MUTE_ALL)
    }

    pub const fn is_music_paused(self) -> bool {
        self.contains(Self::PAUSE_MUSIC)
    }

    pub const fn is_sound_muted(self) -> bool {
        self.contains(Self::SOLO_MUSIC) || self.contains(Self::MUTE_ALL)
    }
}

impl From<ToggleStates> for PlayerStatus {
    fn from(state: ToggleStates) -> Self {
        Self::from_bits(state.bits()).unwrap_or_default()
    }
}

impl From<PlayerStatus> for ToggleStates {
    fn from(state: PlayerStatus) -> Self {
        Self::from_bits(state.bits()).unwrap_or_default()
    }
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            last_refresh: Instant::now(),
            last_local_update: Instant::now(),
            refresh_interval: Duration::from_millis(10),
            local_elapsed: 0,
            status: PlayerStatus::default(),
        }
    }
}
