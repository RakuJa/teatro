use crate::FilterData;
use crate::audio::audio_filter::FilteredSource;
use crate::backend::errors::{FilterError, PlaybackError};
use crate::states::playlist_data::Track;
use biquad::{Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};
use rodio::{Player, Source};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::warn;

/// Sample rate assumed by the filter. (should be derived from source, not hard coded)
const SAMPLE_RATE: f32 = 44_100.;

/// Filter cutoff bounds, as a percentage of the sample rate (50% == Nyquist).
const MIN_FILTER_PERCENTAGE: f32 = 1.;
const MAX_FILTER_PERCENTAGE: f32 = 50.;

/// How much margin before the end when seeking
const END_MARGIN: Duration = Duration::from_millis(250);

/// Seeks the current track to an absolute position. `track_length` is used to ensure that
/// We are not seeking after the song.
pub fn seek_to(
    sink: &Player,
    position: Duration,
    track_length: Duration,
) -> Result<(), PlaybackError> {
    let target = position.min(track_length.saturating_sub(END_MARGIN));
    sink.try_seek(target).map_err(PlaybackError::Seek)
}

/// Seeks to a fraction (0.0..=1.0) representing a percentage of the track instead of an absolute position
pub fn seek_to_fraction(
    sink: &Player,
    fraction: f32,
    total: Duration,
) -> Result<(), PlaybackError> {
    let fraction = if fraction.is_finite() {
        fraction.clamp(0., 1.)
    } else {
        0.
    };
    seek_to(sink, total.mul_f32(fraction), total)
}

/// Moves the filter cutoff by `delta` percentage points and rebuilds the coefficients.
///
/// The stored percentage is clamped to `[1, 50]`, so it can't wind up past
/// the point where the cutoff stops changing.
pub fn change_filter_frequency_value(
    filter: &Arc<Mutex<FilterData>>,
    delta: f32,
    filter_type: Type<f32>,
) -> Result<(), FilterError> {
    let mut data = filter.lock().map_err(|_| FilterError::DataPoisoned)?;

    let perc = (data.previous_filter_percentage + delta)
        .clamp(MIN_FILTER_PERCENTAGE, MAX_FILTER_PERCENTAGE);
    let cutoff = (SAMPLE_RATE / 100. * perc).min(SAMPLE_RATE / 2.);

    let coeffs = Coefficients::<f32>::from_params(
        filter_type,
        SAMPLE_RATE.hz(),
        cutoff.hz(),
        Q_BUTTERWORTH_F32,
    )
    .map_err(FilterError::Coefficients)?;

    *data
        .filter
        .lock()
        .map_err(|_| FilterError::FilterPoisoned)? = DirectForm1::<f32>::new(coeffs);

    data.previous_filter_percentage = perc;
    data.filter_type = filter_type;
    Ok(())
}

/// Sets the volume, clamped to `[0, 1]`, and returns the value actually applied.
pub fn change_volume(sink: &Player, value: f32) -> f32 {
    sink.set_volume(value.clamp(0., 1.));
    sink.volume()
}

/// Adds `value` to the current volume and returns the new volume.
pub fn increase_volume(sink: &Player, value: f32) -> f32 {
    change_volume(sink, sink.volume() + value)
}

/// Decodes `path` and appends it to the sink's queue.
///
/// the track is wrapped in a `FilteredSource`. if the
/// filter lock can't be taken, the track is queued unfiltered.
///
/// This does not start playback.
pub fn enqueue(
    sink: &Player,
    path: &str,
    filter: Option<&Arc<Mutex<FilterData>>>,
) -> Result<Track, PlaybackError> {
    let file = std::fs::File::open(path).map_err(|source| PlaybackError::Open {
        path: path.to_owned(),
        source,
    })?;
    let source = rodio::Decoder::try_from(file).map_err(|source| PlaybackError::Decode {
        path: path.to_owned(),
        source,
    })?;
    let track_length = source.total_duration();

    let shared_filter = filter.and_then(|f| {
        f.lock().map_or_else(
            |_| {
                warn!("Filter lock failed, queueing {path} without a filter");
                None
            },
            |guard| Some(Arc::clone(&guard.filter)),
        )
    });

    match shared_filter {
        Some(filter) => sink.append(FilteredSource { source, filter }),
        None => sink.append(source),
    }

    Ok(Track::builder()
        .track_length(track_length)
        .file_path(path)
        .build())
}

pub fn current_track_index(sink: &Player, total: usize) -> usize {
    total.saturating_sub(remaining_tracks(sink))
}

/// Jumps to `target` in the playlist, forwards or backwards.
///
/// A `Player` queue only moves forward, so this clears it and re-queues `tracks`.
pub fn go_to_track(
    sink: &Player,
    tracks: &[Track],
    target_idx: usize,
    filter: Option<&Arc<Mutex<FilterData>>>,
) -> Result<(), PlaybackError> {
    if target_idx >= tracks.len() {
        return Err(PlaybackError::TrackOutOfRange {
            index: target_idx,
            len: tracks.len(),
        });
    }

    let was_paused = sink.is_paused();
    sink.clear();

    for track in &tracks[target_idx..] {
        if let Err(e) = enqueue(sink, &track.file_path, filter) {
            warn!("Skipping {}: {e}", track.file_path);
        }
    }

    if !was_paused {
        sink.play();
    }
    Ok(())
}

pub fn stop_track(sink: &Player) {
    sink.stop();
}

pub fn pause_track(sink: &Player) {
    sink.pause();
}

pub fn resume_track(sink: &Player) {
    sink.play();
}

/// Number of sources still in the queue (including the one playing).
pub fn remaining_tracks(sink: &Player) -> usize {
    sink.len()
}

pub fn track_elapsed_time(sink: &Player) -> Duration {
    sink.get_pos()
}
