use biquad::{Biquad, DirectForm1};
use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::warn;

pub struct FilteredSource<S> {
    pub(crate) source: S,
    pub(crate) filter: Arc<Mutex<DirectForm1<f32>>>,
}

impl<S> Iterator for FilteredSource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if let Ok(mut filter) = self.filter.lock() {
            let sample = self.source.next()?;
            Some(filter.run(sample))
        } else {
            warn!("Failed to lock filter, could not advance filter iterator");
            None
        }
    }
}

impl<S> Source for FilteredSource<S>
where
    S: Source<Item = f32>,
{
    fn current_span_len(&self) -> Option<usize> {
        self.source.current_span_len()
    }
    fn channels(&self) -> ChannelCount {
        self.source.channels()
    }
    fn sample_rate(&self) -> SampleRate {
        self.source.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.source.try_seek(pos)?;
        if let Ok(mut filter) = self.filter.lock() {
            filter.reset_state();
        } else {
            warn!("Failed to lock filter, could not reset state after seek");
        }
        Ok(())
    }
}
