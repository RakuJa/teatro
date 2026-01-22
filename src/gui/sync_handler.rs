use crate::comms::command::{Command, Device};
use crate::gui::ui::AkaiVisualizer;
use crate::hardware_handler::midi_handler::MidiHandler;
use crate::hardware_handler::pad_handler::PadHandler;
use crate::states::music_state::MusicState;
use crate::states::visualizer::AkaiData;
use flume::{Receiver, Sender};
use log::{debug, warn};
use std::sync::{Arc, Mutex};

pub fn sync_gui_with_data_received_from_backend(
    rx_data: &Receiver<AkaiData>,
    akai_visualizer: &Arc<Mutex<AkaiVisualizer>>,
) {
    loop {
        if let Ok(x) = rx_data.recv() {
            if let Ok(mut visualizer) = akai_visualizer.lock() {
                debug!("{x:?}");
                visualizer.local_elapsed = x
                    .current_playlist
                    .as_ref()
                    .and_then(super::super::states::playlist_data::PlaylistData::get_current_track)
                    .map_or(0, |t| t.elapsed_seconds * 1000);
                visualizer.data = x;
            } else {
                warn!("Couldn't update GUI data. GUI will not update.");
            }
        } else {
            warn!("Failed to receive data from backend. GUI will not update.");
        }
    }
}

pub fn handle_gui_command_and_relay_them_to_backend(
    rx_command: &Receiver<Command>,
    tx_data: &Sender<AkaiData>,
    music_state: &MusicState,
) {
    loop {
        if let Ok(command) = rx_command.try_recv() {
            match command {
                Command::Refresh { device } => match device {
                    Device::ToBackend => {
                        if let Ok(audio_sink) = music_state.audio_sinks.lock()
                            && let Ok(data) = music_state.data.lock()
                        {
                            PadHandler::refresh(&data, tx_data, &audio_sink);
                        }
                    }
                    Device::ToGui => (),
                },
            }
        }
    }
}
