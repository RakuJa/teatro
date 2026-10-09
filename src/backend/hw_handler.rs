use crate::FilterData;
use crate::audio::playback_handler;
use crate::states::audio_sinks::AudioSinks;
use crate::states::playlist_data::{PlaylistData, Track};
use crate::states::visualizer::RuntimeData;
use flume::Sender;
use ramidier::io::input_data::MidiInputData;
use ramidier::io::output::ChannelOutput;
use rodio::Player;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::warn;

pub trait MidiHandler {
    type Group;

    type State;

    /// Recomputes the runtime data (volumes, playlist progress, ...) from the audio sinks.
    fn refresh(
        old_data: &RuntimeData,
        tx_data: &Sender<RuntimeData>,
        audio_sinks: &AudioSinks,
    ) -> RuntimeData;

    /// Pushes a snapshot of `data` to the GUI.
    fn update_gui(tx: &Sender<RuntimeData>, data: &RuntimeData) {
        if tx.send(data.clone()).is_err() {
            warn!("Failed to send data to update GUI: receiver dropped");
        }
    }

    /// Rebuilds the playlist state from the sink: which track is current and how far in it is.
    fn current_playlist_state(old: PlaylistData, sink: &Player) -> PlaylistData {
        let current_track_number = old
            .tracks
            .len()
            .saturating_sub(playback_handler::remaining_tracks(sink));

        let tracks = old
            .tracks
            .into_iter()
            .enumerate()
            .map(|(i, t)| Track {
                elapsed_time: if i == current_track_number {
                    playback_handler::track_elapsed_time(sink)
                } else {
                    Duration::default()
                },
                ..t
            })
            .collect();

        PlaylistData {
            tracks,
            current_track: current_track_number as u64,
        }
    }

    /// Clears the sink, queues every playable file (filter applied to all of them)
    /// and starts playback.
    ///
    /// Returns `None` when `files` is empty. Files that fail to load are skipped
    /// with a warning and are not part of the returned playlist.
    fn play_playlist(
        files: &[String],
        sink: &Player,
        filter: &Arc<Mutex<FilterData>>,
        volume: Option<f32>,
    ) -> Option<PlaylistData> {
        if files.is_empty() {
            return None;
        }

        sink.clear();
        if let Some(v) = volume {
            playback_handler::change_volume(sink, v);
        }

        let tracks: Vec<Track> = files
            .iter()
            .filter_map(|f| match playback_handler::enqueue(sink, f, Some(filter)) {
                Ok(t) => Some(t),
                Err(e) => {
                    warn!("Skipping {f}: {e}");
                    None
                }
            })
            .collect();

        if tracks.is_empty() {
            warn!("None of the {} files could be loaded", files.len());
            return None;
        }

        sink.play();
        Some(PlaylistData::builder().tracks(tracks).build())
    }

    fn listener(
        midi_out: Arc<Mutex<ChannelOutput>>,
        stamp: u64,
        msg: &MidiInputData<Self::Group>,
        state: &mut Self::State,
    );
}
