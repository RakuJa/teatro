use crate::FilterData;
use crate::audio::playback_handler;
use crate::states::visualizer::AkaiData;
use flume::Sender;
use log::warn;
use rodio::Sink;
use std::sync::{Arc, Mutex};

pub fn play_song(files: &[String], sound_queue: &Sink, filter: &Arc<Mutex<FilterData>>) {
    if let Some(first_track) = files.first() {
        let () = sound_queue.clear();
        let _ = playback_handler::play_track(sound_queue, first_track.as_str(), Some(filter));
        files.iter().skip(1).for_each(|file| {
            let _ = playback_handler::add_track_to_queue(sound_queue, file.as_str(), false);
        });
    }
}

pub fn update_gui(tx_channel: &Sender<AkaiData>, data: AkaiData) {
    match tx_channel.send(data) {
        Ok(()) => (),
        _ => warn!("Failed to send data to update GUI"),
    }
}
