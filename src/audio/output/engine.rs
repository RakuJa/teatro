use crate::audio::output::selector::Selector;
use crate::audio::output::tap::Tap;
use crate::audio::output::{CHUNK, OUT_CHANNELS, OUT_RATE, OutputKind};
use rodio::mixer::{Mixer, MixerSource};
use rodio::{ChannelCount, SampleRate};
use std::sync::{Arc, Mutex};

/// Owns the master mix that every `Player` connects to.
#[derive(Clone)]
pub struct Engine {
    mixer: Mixer,
    master: Arc<Mutex<MixerSource>>,
    selector: Selector,
}

impl Engine {
    pub fn new() -> Self {
        let (mixer, source) = rodio::mixer::mixer(
            ChannelCount::new(OUT_CHANNELS as u16)
                .expect("OUT CHANNELS should be different from 0"),
            SampleRate::new(OUT_RATE).expect("OUT RATE should be different from 0"),
        );
        Self {
            mixer,
            master: Arc::new(Mutex::new(source)),
            selector: Selector::new(),
        }
    }

    pub const fn mixer(&self) -> &Mixer {
        &self.mixer
    }
    pub fn selector(&self) -> Selector {
        self.selector.clone()
    }

    pub fn tap(&self, kind: OutputKind) -> Tap {
        Tap {
            master: Arc::clone(&self.master),
            selector: self.selector.clone(),
            kind,
            buf: Vec::with_capacity(CHUNK),
            pos: 0,
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
