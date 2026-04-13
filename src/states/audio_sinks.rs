use rodio::Player;

pub struct AudioSinks {
    pub music_queue: Player,
    pub ambience_queue: Player,
    pub sound_effect_queue: Player,
}
