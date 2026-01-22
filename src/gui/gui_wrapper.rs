use crate::gui::ui::AkaiVisualizer;
use std::sync::{Arc, Mutex};

/// Wrapper used to share gui access
pub struct GuiWrapper {
    pub state: Arc<Mutex<AkaiVisualizer>>,
}

impl eframe::App for GuiWrapper {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(mut state) = self.state.lock() {
            state.update(ctx, frame);
        }
        ctx.request_repaint();
    }
}
