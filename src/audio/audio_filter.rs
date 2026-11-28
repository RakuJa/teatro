use crate::states::filter_data::{FilterChain, FilterData, FilterKind};
use biquad::{Biquad, Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};
use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};
use std::time::Duration;
use tracing::warn;

const MIN_CUTOFF_HZ: f32 = 10.0;
/// Biquads become unstable as the cutoff approaches Nyquist
/// (poles land on the unit circle at exactly `fs / 2`)
const MAX_CUTOFF_RATIO: f32 = 0.45;

fn coefficients(
    kind: FilterKind,
    cutoff_hz: f32,
    sample_rate: f32,
) -> Result<Coefficients<f32>, biquad::Errors> {
    let cutoff = cutoff_hz.clamp(MIN_CUTOFF_HZ, sample_rate * MAX_CUTOFF_RATIO);
    let filter_type = match kind {
        FilterKind::LowPass => Type::LowPass,
        FilterKind::HighPass => Type::HighPass,
        FilterKind::SoftLowPass => Type::SinglePoleLowPass,
    };
    Coefficients::<f32>::from_params(
        filter_type,
        sample_rate.hz(),
        cutoff.hz(),
        Q_BUTTERWORTH_F32,
    )
}

struct Stage {
    kind: FilterKind,
    per_channel: Vec<DirectForm1<f32>>,
}

pub struct FilteredSource<S> {
    source: S,
    data: FilterData,
    /// Active stages in processing order. Empty means passthrough.
    stages: Vec<Stage>,
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
            stages: Vec::new(),
            // 0 forces a build on the first sample
            channels: 0,
            channel: 0,
            sample_rate: 0,
            seen_version: 0,
        }
    }

    /// Rebuilds the stages from the shared chain. When the format and the set
    /// of active stages are unchanged only the coefficients are swapped, so
    /// sweeping a knob doesn't reset the filter state.
    fn sync(&mut self, format_changed: bool) {
        self.seen_version = self.data.version();
        let chain: FilterChain = self.data.chain();
        let rate = self.sample_rate as f32;

        let wanted: Vec<(FilterKind, Coefficients<f32>)> = chain
            .stages()
            .filter_map(|(kind, hz)| match coefficients(kind, hz, rate) {
                Ok(c) => Some((kind, c)),
                Err(e) => {
                    warn!("Skipping {kind:?} filter at {hz} Hz ({rate} Hz audio): {e:?}");
                    None
                }
            })
            .collect();

        let same_layout = !format_changed
            && self.stages.len() == wanted.len()
            && self
                .stages
                .iter()
                .zip(&wanted)
                .all(|(s, (k, _))| s.kind == *k && s.per_channel.len() == self.channels);

        if same_layout {
            for (stage, (_, c)) in self.stages.iter_mut().zip(wanted) {
                stage
                    .per_channel
                    .iter_mut()
                    .for_each(|f| f.update_coefficients(c));
            }
        } else {
            self.stages = wanted
                .into_iter()
                .map(|(kind, c)| Stage {
                    kind,
                    per_channel: vec![DirectForm1::new(c); self.channels],
                })
                .collect();
        }
    }

    fn reset_state(&mut self) {
        for stage in &mut self.stages {
            stage.per_channel.iter_mut().for_each(Biquad::reset_state);
        }
    }

    fn process(&mut self, sample: f32) -> f32 {
        let mut out = sample;
        for stage in &mut self.stages {
            if let Some(f) = stage.per_channel.get_mut(self.channel) {
                out = f.run(out);
            }
        }
        if out.is_finite() {
            out
        } else {
            // Never let a NaN/inf get stuck in the filter memory.
            self.reset_state();
            sample
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

        let out = self.process(sample);
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
        self.reset_state();
        self.channel = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;

    const RATE: u32 = 48_000;

    fn sine(hz: f32, millis: usize) -> SamplesBuffer {
        let n = RATE as usize * millis / 1_000;
        let data: Vec<f32> = (0..n)
            .map(|i| (std::f32::consts::TAU * hz * i as f32 / RATE as f32).sin() * 0.5)
            .collect();
        SamplesBuffer::new(
            ChannelCount::new(1).expect("non-zero"),
            SampleRate::new(RATE).expect("non-zero"),
            data,
        )
    }

    fn rms_tail(samples: &[f32]) -> f32 {
        let tail = &samples[samples.len() / 2..];
        (tail.iter().map(|s| s * s).sum::<f32>() / tail.len() as f32).sqrt()
    }

    fn run(hz: f32, chain: FilterChain) -> Vec<f32> {
        let data = FilterData::new();
        data.set(chain);
        FilteredSource::new(sine(hz, 200), data).collect()
    }

    #[test]
    fn bypassed_chain_is_bit_exact_passthrough() {
        let dry: Vec<f32> = sine(440.0, 200).collect();
        let wet = run(440.0, FilterChain::default());
        assert_eq!(dry, wet);
    }

    #[test]
    fn low_pass_cuts_highs_and_keeps_lows() {
        let chain = FilterChain {
            low_pass: Some(500.0),
            ..FilterChain::default()
        };
        let dry = rms_tail(&run(100.0, FilterChain::default()));
        assert!(rms_tail(&run(100.0, chain)) > dry * 0.9);
        assert!(rms_tail(&run(8_000.0, chain)) < dry * 0.05);
    }

    #[test]
    fn high_pass_cuts_lows() {
        let chain = FilterChain {
            high_pass: Some(2_000.0),
            ..FilterChain::default()
        };
        let dry = rms_tail(&run(100.0, FilterChain::default()));
        assert!(rms_tail(&run(100.0, chain)) < dry * 0.05);
    }

    #[test]
    fn extreme_cutoffs_stay_finite() {
        for hz in [0.0, 1.0, 23_999.0, 24_000.0, 1e9] {
            for chain in [
                FilterChain {
                    low_pass: Some(hz),
                    ..FilterChain::default()
                },
                FilterChain {
                    high_pass: Some(hz),
                    ..FilterChain::default()
                },
                FilterChain {
                    soft_low_pass: Some(hz),
                    ..FilterChain::default()
                },
            ] {
                assert!(
                    run(1_000.0, chain).iter().all(|s| s.is_finite()),
                    "{chain:?}"
                );
            }
        }
    }

    #[test]
    fn sweeping_while_playing_stays_finite_and_recovers() {
        let data = FilterData::new();
        let mut src = FilteredSource::new(sine(1_000.0, 1_000), data.clone());
        for step in 0..=200 {
            let v = step as f32 / 200.0;
            data.set(FilterChain {
                high_pass: FilterKind::HighPass.cutoff_hz(v),
                low_pass: FilterKind::LowPass.cutoff_hz(v),
                soft_low_pass: FilterKind::SoftLowPass.cutoff_hz(1.0 - v),
            });
            for _ in 0..100 {
                assert!(src.next().is_some_and(f32::is_finite));
            }
        }
        data.set(FilterChain::default());
        let dry: Vec<f32> = sine(1_000.0, 1_000).skip(20_100).take(100).collect();
        let wet: Vec<f32> = src.by_ref().take(100).collect();
        assert_eq!(dry, wet);
    }
}
