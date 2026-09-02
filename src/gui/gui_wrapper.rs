use crate::gui::ui::AkaiVisualizer;
use egui::Ui;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;

/// Wrapper used to share gui access
pub struct GuiWrapper {
    pub state: Rc<Mutex<AkaiVisualizer>>,
}

impl eframe::App for GuiWrapper {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        #[cfg(feature = "profiling")]
        puffin_egui::puffin::GlobalProfiler::lock().new_frame();

        let display_refresh_rate = self.state.lock().map_or(60.0, |mut state| {
            state.ui(ui, frame);
            state.settings_data.repaint_display_hz
        });
        let is_minimized = ui.input(|i| i.viewport().minimized.unwrap_or(false));
        if !is_minimized {
            let is_focused = ui.input(|i| i.viewport().focused.unwrap_or(true));
            ui.request_repaint_after(if is_focused {
                Duration::from_secs_f32(1.0 / display_refresh_rate.max(1.0))
            } else {
                Duration::from_millis(250) // ~4fps,
            });
        }

        #[cfg(feature = "profiling")]
        puffin_egui::profiler_window(ui.ctx());
    }
}
