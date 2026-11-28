use crate::FilterData;
use crate::audio::audio_filter::FilteredSource;
use crate::audio::looping_source::{LoopControl, LoopingSource};
use crate::backend::errors::PlaybackError;
use crate::states::playlist_data::Track;
use rodio::{Player, Source};
use std::time::Duration;
use tracing::warn;

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
/// With `looping`, the track repeats while that control is enabled.
///
/// This does not start playback.
pub fn enqueue(
    sink: &Player,
    path: &str,
    filter: Option<&FilterData>,
    looping: Option<&LoopControl>,
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

    // The loop sits inside the filter so the filter state carries across the loop.
    match looping {
        Some(control) => append_filtered(
            sink,
            LoopingSource::new(source, path.to_owned(), control.clone()),
            filter,
        ),
        None => append_filtered(sink, source, filter),
    }

    Ok(Track::builder()
        .maybe_track_length(track_length)
        .file_path(path)
        .build())
}

fn append_filtered<S>(sink: &Player, source: S, filter: Option<&FilterData>)
where
    S: Source<Item = f32> + Send + 'static,
{
    match filter {
        Some(f) => sink.append(FilteredSource::new(source, f.clone())),
        None => sink.append(source),
    }
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
    filter: Option<&FilterData>,
    looping: Option<&LoopControl>,
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
        if let Err(e) = enqueue(sink, &track.file_path, filter, looping) {
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

pub fn track_elapsed_time(sink: &Player, looped: Duration) -> Duration {
    sink.get_pos().saturating_sub(looped)
}
