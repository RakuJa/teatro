use rodio::source::SeekError;
use rodio::{ChannelCount, Decoder, SampleRate, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tracing::warn;

type FileDecoder = Decoder<BufReader<File>>;

#[derive(Default)]
struct Shared {
    enabled: AtomicBool,
    looped_nanos: AtomicU64,
}

#[derive(Clone, Default)]
pub struct LoopControl(Arc<Shared>);

impl LoopControl {
    pub fn set_enabled(&self, enabled: bool) {
        self.0.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.0.enabled.load(Ordering::Relaxed)
    }

    pub fn time_spent_in_loop(&self) -> Duration {
        Duration::from_nanos(self.0.looped_nanos.load(Ordering::Relaxed))
    }

    fn add_looped(&self, pass: Duration) {
        let nanos = u64::try_from(pass.as_nanos()).unwrap_or(u64::MAX);
        self.0.looped_nanos.fetch_add(nanos, Ordering::Relaxed);
    }

    fn reset_looped(&self) {
        self.0.looped_nanos.store(0, Ordering::Relaxed);
    }
}

pub struct LoopingSource {
    inner: FileDecoder,
    path: String,
    control: LoopControl,
    started: bool,
    samples_this_pass: u64,
}

impl LoopingSource {
    pub const fn new(inner: FileDecoder, path: String, control: LoopControl) -> Self {
        Self {
            inner,
            path,
            control,
            started: false,
            samples_this_pass: 0,
        }
    }

    fn samples_per_second(&self) -> f64 {
        f64::from(self.inner.sample_rate().get()) * f64::from(self.inner.channels().get())
    }

    fn rewind(&mut self) -> bool {
        if self.inner.try_seek(Duration::ZERO).is_ok() {
            return true;
        }
        let reopened = File::open(&self.path)
            .map_err(|e| e.to_string())
            .and_then(|f| Decoder::try_from(f).map_err(|e| e.to_string()));
        match reopened {
            Ok(decoder) => {
                self.inner = decoder;
                true
            }
            Err(e) => {
                warn!("Cannot loop {}: {e}", self.path);
                false
            }
        }
    }
}

impl Iterator for LoopingSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if !self.started {
            self.started = true;
            self.control.reset_looped();
        }

        if let Some(sample) = self.inner.next() {
            self.samples_this_pass += 1;
            return Some(sample);
        }

        if !self.control.is_enabled() || self.samples_this_pass == 0 {
            return None;
        }
        let pass =
            Duration::from_secs_f64(self.samples_this_pass as f64 / self.samples_per_second());
        if !self.rewind() {
            return None;
        }
        self.control.add_looped(pass);
        self.samples_this_pass = 0;

        let sample = self.inner.next()?;
        self.samples_this_pass = 1;
        Some(sample)
    }
}

impl Source for LoopingSource {
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }
    fn channels(&self) -> ChannelCount {
        self.inner.channels()
    }
    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.inner.try_seek(pos)?;
        self.control.reset_looped();
        let per_second =
            u128::from(self.inner.sample_rate().get()) * u128::from(self.inner.channels().get());
        self.samples_this_pass =
            u64::try_from(pos.as_nanos() * per_second / 1_000_000_000).unwrap_or(u64::MAX);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn wav(name: &str, samples: &[i16]) -> std::path::PathBuf {
        let path =
            std::env::temp_dir().join(format!("teatro-loop-{name}-{}.wav", std::process::id()));
        let data_len = u32::try_from(samples.len() * 2).expect("small test file");
        let mut f = File::create(&path).expect("create wav");
        let rate: u32 = 8_000;
        let mut header = Vec::new();
        header.extend_from_slice(b"RIFF");
        header.extend_from_slice(&(36 + data_len).to_le_bytes());
        header.extend_from_slice(b"WAVEfmt ");
        header.extend_from_slice(&16u32.to_le_bytes());
        header.extend_from_slice(&1u16.to_le_bytes());
        header.extend_from_slice(&1u16.to_le_bytes());
        header.extend_from_slice(&rate.to_le_bytes());
        header.extend_from_slice(&(rate * 2).to_le_bytes());
        header.extend_from_slice(&2u16.to_le_bytes());
        header.extend_from_slice(&16u16.to_le_bytes());
        header.extend_from_slice(b"data");
        header.extend_from_slice(&data_len.to_le_bytes());
        f.write_all(&header).expect("write header");
        for s in samples {
            f.write_all(&s.to_le_bytes()).expect("write sample");
        }
        path
    }

    fn looping(path: &std::path::Path, control: &LoopControl) -> LoopingSource {
        let decoder = Decoder::try_from(File::open(path).expect("open wav")).expect("decode wav");
        LoopingSource::new(decoder, path.display().to_string(), control.clone())
    }

    const SAMPLES: [i16; 400] = [1_000; 400];

    #[test]
    fn plays_once_when_disabled() {
        let path = wav("once", &SAMPLES);
        let control = LoopControl::default();
        assert_eq!(looping(&path, &control).count(), SAMPLES.len());
        assert_eq!(control.time_spent_in_loop(), Duration::ZERO);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn repeats_while_enabled_and_tracks_looped_time() {
        let path = wav("repeat", &SAMPLES);
        let control = LoopControl::default();
        control.set_enabled(true);
        let mut src = looping(&path, &control);

        assert_eq!(
            src.by_ref().take(SAMPLES.len() * 3 + 10).count(),
            SAMPLES.len() * 3 + 10
        );
        assert_eq!(control.time_spent_in_loop(), Duration::from_millis(150));

        // Turning looping off lets the current pass finish, then it ends.
        control.set_enabled(false);
        assert_eq!(src.count(), SAMPLES.len() - 10);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn player_queue_stays_on_the_looping_track() {
        use crate::audio::playback_handler::{enqueue, remaining_tracks, track_elapsed_time};

        let first = wav("player-a", &[1_000; 4_000]);
        let second = wav("player-b", &[2_000; 4_000]);
        let control = LoopControl::default();
        control.set_enabled(true);

        let (player, mut output) = rodio::Player::new();
        for path in [&first, &second] {
            enqueue(&player, path.to_str().expect("utf-8"), None, Some(&control)).expect("enqueue");
        }
        player.play();
        assert_eq!(remaining_tracks(&player), 2);

        let pulled = 4_000 * 9 / 4;
        assert_eq!(output.by_ref().take(pulled).count(), pulled);

        assert_eq!(remaining_tracks(&player), 2, "queue advanced while looping");
        let elapsed = track_elapsed_time(&player, control.time_spent_in_loop());
        assert!(
            elapsed >= Duration::from_millis(100) && elapsed <= Duration::from_millis(130),
            "elapsed {elapsed:?}"
        );
        assert!(
            output
                .next()
                .is_some_and(|s| (s - 1_000.0 / 32_768.0).abs() < 1e-3)
        );

        control.set_enabled(false);
        let rest = 4_000 - pulled % 4_000;
        assert_eq!(output.by_ref().take(rest + 10).count(), rest + 10);
        assert_eq!(remaining_tracks(&player), 1);

        let _ = std::fs::remove_file(first);
        let _ = std::fs::remove_file(second);
    }

    #[test]
    fn seeking_resets_the_looped_time() {
        let path = wav("seek", &SAMPLES);
        let control = LoopControl::default();
        control.set_enabled(true);
        let mut src = looping(&path, &control);
        assert_eq!(
            src.by_ref().take(SAMPLES.len() + 5).count(),
            SAMPLES.len() + 5
        );
        assert!(control.time_spent_in_loop() > Duration::ZERO);

        src.try_seek(Duration::from_millis(25))
            .expect("wav is seekable");
        assert_eq!(control.time_spent_in_loop(), Duration::ZERO);
        assert_eq!(
            src.by_ref().take(SAMPLES.len() - 200 + 1).count(),
            SAMPLES.len() - 200 + 1
        );
        assert_eq!(control.time_spent_in_loop(), Duration::from_millis(50));
        let _ = std::fs::remove_file(path);
    }
}
