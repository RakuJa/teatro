use crate::audio::looping_source::LoopControl;
use rodio::Player;

pub struct AudioSinks {
    pub music_queue: Player,
    pub music_loop: LoopControl,
    pub ambience_queue: Player,
    pub sound_effect_queue: Player,
}
