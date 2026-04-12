use crate::gui::ui::AkaiVisualizer;
use egui::Ui;
use std::rc::Rc;
use std::sync::Mutex;

/// Wrapper used to share gui access
pub struct GuiWrapper {
    pub state: Rc<Mutex<AkaiVisualizer>>,
}

impl eframe::App for GuiWrapper {
    fn ui(&mut self, _: &mut Ui, _: &mut eframe::Frame) {}

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(mut state) = self.state.lock() {
            state.update(ctx, frame);
        }
        ctx.request_repaint();
    }
}
