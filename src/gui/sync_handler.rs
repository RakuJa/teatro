use crate::gui::handler::AkaiVisualizer;
use crate::states::visualizer::AkaiData;
use flume::Receiver;
use log::warn;
use std::sync::{Arc, Mutex};

pub fn sync_gui_with_hardware(
    rx_data: &Receiver<AkaiData>,
    akai_visualizer: &Arc<Mutex<AkaiVisualizer>>,
) {
    loop {
        if let Ok(x) = rx_data.recv() {
            if let Ok(mut visualizer) = akai_visualizer.lock() {
                visualizer.data = x;
            } else {
                warn!("Couldn't update GUI data. GUI will not update.");
            }
        } else {
            warn!("Failed to receive data from backend. GUI will not update.");
        }
    }
}
