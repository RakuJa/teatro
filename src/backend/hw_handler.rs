use crate::FilterData;
use crate::audio::playback_handler;
use crate::states::audio_sinks::AudioSinks;
use crate::states::playlist_data::{PlaylistData, Track};
use crate::states::visualizer::RuntimeData;
use flume::Sender;
use log::warn;
use ramidier::io::input_data::MidiInputData;
use ramidier::io::output::ChannelOutput;
use rodio::Sink;
use std::sync::{Arc, Mutex};

pub trait MidiHandler {
    type Group;

    type State;

    fn refresh(
        old_data: &RuntimeData,
        tx_data: &Sender<RuntimeData>,
        audio_sinks: &AudioSinks,
    ) -> RuntimeData;

    fn update_gui(tx_channel: &Sender<RuntimeData>, data: &RuntimeData) {
        match tx_channel.send(data.clone()) {
            Ok(()) => (),
            _ => warn!("Failed to send data to update GUI"),
        }
    }

    fn get_current_playlist_state(old_state: PlaylistData, sink: &Sink) -> PlaylistData {
        let curr_track_number =
            old_state.tracks.len() as u64 - playback_handler::get_n_of_remaining_tracks(sink);
        PlaylistData {
            tracks: old_state
                .tracks
                .into_iter()
                .enumerate()
                .map(|(i, x)| Track {
                    file_path: x.file_path.clone(),
                    track_length: x.track_length,
                    elapsed_seconds: if i == curr_track_number as usize {
                        playback_handler::get_current_track_elapsed_time(sink)
                    } else {
                        0
                    },
                })
                .collect(),
            current_track: curr_track_number,
        }
    }

    fn play_song(
        files: &[String],
        sound_queue: &Sink,
        filter: &Arc<Mutex<FilterData>>,
        volume: Option<f32>,
    ) -> Option<PlaylistData> {
        let mut tracks = vec![];
        files.first().map(|first_track| {
            let () = sound_queue.clear();
            if let Some(v) = volume {
                playback_handler::change_volume(sound_queue, v);
            }
            if let Ok(track) =
                playback_handler::play_track(sound_queue, first_track.as_str(), Some(filter))
            {
                tracks.push(track);
            }
            files.iter().skip(1).for_each(|file| {
                if let Ok(track) =
                    playback_handler::add_track_to_queue(sound_queue, file.as_str(), false)
                {
                    tracks.push(track);
                }
            });
            PlaylistData::builder().tracks(tracks).build()
        })
    }

    fn listener(
        midi_out: Arc<Mutex<ChannelOutput>>,
        stamp: u64,
        msg: &MidiInputData<Self::Group>,
        state: &mut Self::State,
    );
}
