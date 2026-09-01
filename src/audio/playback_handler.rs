use crate::FilterData;
use crate::audio::audio_filter::FilteredSource;
use crate::states::playlist_data::Track;
use biquad::{Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};
use rodio::{Player, Source};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tracing::warn;

pub fn change_filter_frequency_value(
    filter: &Arc<Mutex<FilterData>>,
    value: f32,
    filter_type: Type<f32>,
) {
    if let Ok(mut data) = filter.lock() {
        let fs = 44100.;
        let next_perc = if data.previous_filter_percentage + value <= 1. {
            1.
        } else {
            data.previous_filter_percentage + value
        };
        let f_val = fs / 100. * next_perc;
        if let Ok(coeffs) = Coefficients::<f32>::from_params(
            filter_type,
            fs.hz(),
            if f_val < fs / 2. { f_val } else { fs / 2. }.hz(),
            Q_BUTTERWORTH_F32,
        ) {
            data.previous_filter_percentage = next_perc;
            data.filter_type = filter_type;
            if let Ok(mut f) = data.filter.lock() {
                *f = DirectForm1::<f32>::new(coeffs);
            } else {
                warn!("Failed to get FilterData, cannot change filter frequency");
            }
        } else {
            warn!("Failed to get coeffs to change filter value, cannot change filter frequency");
        }
    } else {
        warn!("Failed to get filter data lock, cannot change filter frequency")
    }
}

pub fn change_volume(sink: &Player, value: f32) -> f32 {
    sink.set_volume(if value <= 0. { 0. } else { value.min(1.) });
    sink.volume()
}

pub fn increase_volume(sink: &Player, value: f32) -> f32 {
    change_volume(sink, sink.volume() + value)
}

pub fn add_track_to_queue(
    sink: &Player,
    file_path: &str,
    play: bool,
) -> Result<Track, Box<dyn Error>> {
    let file = std::fs::File::open(file_path)?;
    let source = rodio::Decoder::try_from(file)?;
    let track_length = source.total_duration();
    sink.append(source);
    if play {
        sink.play();
    }
    Ok(Track::builder()
        .track_length(track_length)
        .file_path(file_path)
        .build())
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

pub fn play_track(
    sink: &Player,
    file_path: &str,
    filter: Option<&Arc<Mutex<FilterData>>>,
) -> Result<Track, Box<dyn Error>> {
    sink.stop();
    sink.clear();

    let file = std::fs::File::open(file_path)?;
    let source = rodio::Decoder::try_from(file)?;
    let track_length = source.total_duration();
    if let Some(filter) = filter {
        if let Ok(f) = filter.lock() {
            sink.append(FilteredSource {
                source,
                filter: Arc::clone(&f.filter),
            });
        } else {
            warn!("Failed to get filter lock, will not apply filter")
        }
    } else {
        sink.append(source);
    }
    sink.play();
    Ok(Track::builder()
        .track_length(track_length)
        .file_path(file_path)
        .build())
}

pub fn get_n_of_remaining_tracks(sink: &Player) -> u64 {
    sink.len() as u64
}

pub fn get_current_track_elapsed_time(sink: &Player) -> u64 {
    sink.get_pos().as_secs()
}
