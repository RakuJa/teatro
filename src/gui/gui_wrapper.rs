use crate::gui::ui::AkaiVisualizer;
use egui::Ui;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;

const UNFOCUSED_REPAINT: Duration = Duration::from_millis(250);

/// Wrapper used to share gui access
pub struct GuiWrapper {
    pub state: Rc<Mutex<AkaiVisualizer>>,
}

impl eframe::App for GuiWrapper {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        #[cfg(feature = "profiling")]
        puffin_egui::puffin::GlobalProfiler::lock().new_frame();

        let (display_refresh_rate, is_playing) =
            self.state.lock().map_or((60.0, false), |mut state| {
                state.ui(ui, frame);
                let is_playing = state
                    .gui_data
                    .lock()
                    .is_ok_and(|d| d.player_info.is_playing() && d.data.current_playlist.is_some());
                (state.settings_data.repaint_display_hz, is_playing)
            });

        let is_minimized = ui.input(|i| i.viewport().minimized.unwrap_or(false));
        if is_playing && !is_minimized {
            let is_focused = ui.input(|i| i.viewport().focused.unwrap_or(true));
            ui.request_repaint_after(if is_focused {
                Duration::from_secs_f32(1.0 / display_refresh_rate.max(1.0))
            } else {
                UNFOCUSED_REPAINT
            });
        }

        #[cfg(feature = "profiling")]
        puffin_egui::profiler_window(ui.ctx());
    }
}
