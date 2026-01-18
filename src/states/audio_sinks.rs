use rodio::Sink;

pub struct AudioSinks {
    pub music_queue: Sink,
    pub ambience_queue: Sink,
    pub sound_effect_queue: Sink,
}
