use crate::gui::ui::GuiData;
use crate::states::button_states::ToggleStates;
use crate::states::playlist_data::PlaylistData;
use crate::states::visualizer::RuntimeData;
use flume::Receiver;
use log::{debug, warn};
use std::sync::{Arc, Mutex};

pub fn sync_gui_with_data_received_from_backend(
    rx_data: &Receiver<RuntimeData>,
    akai_visualizer: &Arc<Mutex<GuiData>>,
) {
    loop {
        if let Ok(x) = rx_data.recv() {
            if let Ok(mut visualizer) = akai_visualizer.lock() {
                debug!("{x:?}");
                visualizer.audio_player_states.local_elapsed = x
                    .current_playlist
                    .as_ref()
                    .and_then(PlaylistData::get_current_track)
                    .map_or(0, |t| t.elapsed_seconds * 1000);
                visualizer.audio_player_states.mute_on =
                    x.button_states.contains(ToggleStates::MUTE);
                visualizer.audio_player_states.shuffle_on =
                    x.button_states.contains(ToggleStates::SEND);
                visualizer.audio_player_states.loop_on =
                    x.button_states.contains(ToggleStates::SELECT);
                visualizer.audio_player_states.pause_on =
                    x.button_states.contains(ToggleStates::CLIP_STOP);
                visualizer.data = x;
            } else {
                warn!("Couldn't update GUI data. GUI will not update.");
            }
        } else {
            warn!("Failed to receive data from backend. GUI will not update.");
        }
    }
}
