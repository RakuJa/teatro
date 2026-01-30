use crate::gui::local_view::audio_player_states::PlayerStatus;
use crate::gui::ui::GuiData;
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
                visualizer.player_info.local_elapsed = x
                    .current_playlist
                    .as_ref()
                    .and_then(PlaylistData::get_current_track)
                    .map_or(0, |t| t.elapsed_seconds * 1000);
                visualizer.player_info.status = PlayerStatus::from(x.button_states);
                visualizer.data = x;
            } else {
                warn!("Couldn't update GUI data. GUI will not update.");
            }
        } else {
            warn!("Failed to receive data from backend. GUI will not update.");
        }
    }
}
