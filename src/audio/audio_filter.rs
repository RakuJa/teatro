use crate::states::filter_data::{FilterData, FilterParams};
use biquad::{Biquad, Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz};
use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};
use std::time::Duration;
use tracing::warn;

fn coefficients(p: FilterParams, sample_rate: f32) -> Result<Coefficients<f32>, biquad::Errors> {
    let cutoff = (sample_rate / 100. * p.percentage).min(sample_rate / 2.);
    Coefficients::<f32>::from_params(
        p.filter_type,
        sample_rate.hz(),
        cutoff.hz(),
        Q_BUTTERWORTH_F32,
    )
}

pub struct FilteredSource<S> {
    source: S,
    data: FilterData,
    /// One filter per channel. Empty means passthrough (coefficients were invalid).
    filters: Vec<DirectForm1<f32>>,
    channels: usize,
    channel: usize,
    sample_rate: u32,
    seen_version: u64,
}

impl<S: Source<Item = f32>> FilteredSource<S> {
    pub const fn new(source: S, data: FilterData) -> Self {
        Self {
            source,
            data,
            filters: Vec::new(),
            // 0 forces a build on the first sample
            channels: 0,
            channel: 0,
            sample_rate: 0,
            seen_version: 0,
        }
    }

    /// On a format change the filters are rebuilt from scratch
    /// otherwise only the coefficients are swapped, keeping the filter state.
    fn sync(&mut self, format_changed: bool) {
        self.seen_version = self.data.version();
        let params = self.data.params();

        match coefficients(params, self.sample_rate as f32) {
            Ok(c) if format_changed || self.filters.len() != self.channels => {
                self.filters = vec![DirectForm1::new(c); self.channels];
            }
            Ok(c) => self
                .filters
                .iter_mut()
                .for_each(|f| f.update_coefficients(c)),
            Err(e) => {
                warn!(
                    "Invalid filter at {} Hz: {e:?}; passing audio through",
                    self.sample_rate
                );
                self.filters.clear();
            }
        }
    }
}

impl<S: Source<Item = f32>> Iterator for FilteredSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.source.next()?;

        // Query after pulling the sample so span changes are already visible.
        let channels = self.source.channels().get() as usize;
        let rate = self.source.sample_rate().get();
        if channels != self.channels || rate != self.sample_rate {
            self.channels = channels;
            self.sample_rate = rate;
            self.channel = 0;
            self.sync(true);
        } else if self.data.version() != self.seen_version {
            self.sync(false);
        }

        let out = self
            .filters
            .get_mut(self.channel)
            .map_or(sample, |f| f.run(sample));
        self.channel += 1;
        if self.channel >= self.channels {
            self.channel = 0;
        }
        Some(out)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.source.size_hint()
    }
}

impl<S: Source<Item = f32>> Source for FilteredSource<S> {
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
        self.filters.iter_mut().for_each(Biquad::reset_state);
        self.channel = 0;
        Ok(())
    }
}
