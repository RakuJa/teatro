use crate::audio::playback_handler;
use crate::backend::hw_handler::MidiHandler;
use crate::states::audio_sinks::AudioSinks;
use crate::states::music_state::MusicState;
use crate::states::visualizer::RuntimeData;
use flume::Sender;
use ramidier::io::input_data::MidiInputData;
use ramidier::io::output::ChannelOutput;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, warn};

#[derive(Debug, Clone, Copy)]
pub enum ExtraMidiGroup {
    /// Jump to the track at this zero-based index in the current playlist.
    SkipToIndex(usize),
    /// Skip song to given percentage (0. to 1) e.g. 0.5 = skip to half duration
    SeekTo(f32),
}

#[derive(Debug)]
pub struct ExtraMidiHandler;

impl MidiHandler for ExtraMidiHandler {
    type Group = ExtraMidiGroup;

    type State = MusicState;

    fn refresh(
        stale_data: &RuntimeData,
        _tx_data: &Sender<RuntimeData>,
        _audio_sinks: &AudioSinks,
    ) -> RuntimeData {
        stale_data.clone()
    }

    fn listener(
        _midi_out: Arc<Mutex<ChannelOutput>>,
        stamp: u64,
        msg: &MidiInputData<Self::Group>,
        state: &mut Self::State,
    ) {
        debug!("{stamp}: {msg:?}");
        let group = msg.input_group;
        if msg.value != 0 {
            Self::handle_input(group, state);
        }
    }
}

impl ExtraMidiHandler {
    pub fn handle_input(input_group: ExtraMidiGroup, state: &MusicState) {
        match input_group {
            ExtraMidiGroup::SkipToIndex(index) => Self::skip_to_index(index, state),
            ExtraMidiGroup::SeekTo(percentage) => Self::seek_fraction(percentage, state),
        }
    }

    fn current_track_length(state: &MusicState) -> Option<Duration> {
        let playlist = state.data.lock().ok()?.current_playlist.as_ref()?.clone();
        Some(
            playlist
                .tracks
                .get(playlist.current_track as usize)?
                .track_length,
        )
    }

    fn seek_fraction(fraction: f32, state: &MusicState) {
        let Some(total) = Self::current_track_length(state) else {
            debug!("Track length unknown; ignoring seek");
            return;
        };

        let Ok(sinks) = state.audio_sinks.lock() else {
            warn!("Failed to lock audio sinks; cannot seek");
            return;
        };
        if let Err(e) = playback_handler::seek_to_fraction(&sinks.music_queue, fraction, total) {
            warn!("Cannot seek: {e}");
            return;
        }
        drop(sinks);
    }

    fn skip_to_index(index: usize, state: &MusicState) {
        let tracks = {
            let Ok(data) = state.data.lock() else {
                warn!("Failed to lock data; ignoring skip to {index}");
                return;
            };
            let Some(playlist) = data.current_playlist.as_ref() else {
                debug!("No playlist loaded; ignoring skip to {index}");
                return;
            };
            playlist.tracks.clone()
        };

        let Ok(sinks) = state.audio_sinks.lock() else {
            warn!("Failed to lock audio sinks; cannot skip to {index}");
            return;
        };
        if let Err(e) = playback_handler::go_to_track(
            &sinks.music_queue,
            &tracks,
            index,
            Some(&state.music_filter),
        ) {
            warn!("Cannot skip to track {index}: {e}");
            return;
        }
        playback_handler::current_track_index(&sinks.music_queue, tracks.len());
    }
}
