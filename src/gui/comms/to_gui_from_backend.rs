use crate::gui::local_view::audio_player_states::PlayerStatus;
use crate::gui::ui::GuiData;
use crate::states::visualizer::RuntimeData;
use flume::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::warn;

/// Applies every snapshot sent by the backend to the GUI state and wakes the UI up
pub fn sync_gui_with_data_received_from_backend(
    rx_data: &Receiver<RuntimeData>,
    akai_visualizer: &Arc<Mutex<GuiData>>,
    ctx: &egui::Context,
) {
    while let Ok(x) = rx_data.recv() {
        if let Ok(mut visualizer) = akai_visualizer.lock() {
            visualizer.player_info.local_elapsed = x
                .current_playlist
                .as_ref()
                .and_then(|p| p.tracks.get(p.current_track as usize))
                .map_or_else(Duration::default, |t| t.elapsed_time);
            visualizer.player_info.elapsed_synced_at = Instant::now();
            visualizer.player_info.status = PlayerStatus::from(x.button_states);
            visualizer.data = x;
        } else {
            warn!("Couldn't update GUI data. GUI will not update.");
        }
        ctx.request_repaint();
    }
    warn!("Backend data channel closed. GUI will not update anymore.");
}
