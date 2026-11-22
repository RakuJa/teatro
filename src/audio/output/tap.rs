use crate::audio::output::selector::Selector;
use crate::audio::output::{CHUNK, OUT_CHANNELS, OUT_RATE, OutputKind};
use rodio::mixer::MixerSource;
use rodio::{ChannelCount, SampleRate, Source};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct Tap {
    pub(crate) master: Arc<Mutex<MixerSource>>,
    pub(crate) selector: Selector,
    pub(crate) kind: OutputKind,
    pub(crate) buf: Vec<f32>,
    pub(crate) pos: usize,
}

impl Tap {
    fn refill(&mut self) {
        self.buf.clear();
        self.pos = 0;
        if self.selector.active() == self.kind {
            let mut master = self
                .master
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.buf.extend(master.by_ref().take(CHUNK));
            self.buf.resize(CHUNK, 0.);
        } else {
            self.buf.resize(CHUNK, 0.);
        }
    }
}

impl Iterator for Tap {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.buf.len() {
            self.refill();
        }
        let s = self.buf[self.pos];
        self.pos += 1;
        Some(s)
    }
}

impl Source for Tap {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> ChannelCount {
        ChannelCount::new(OUT_CHANNELS as u16).unwrap()
    }
    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(OUT_RATE).expect("OUT RATE should be different from 0")
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
