use crate::gui::comms::command::Command;
use crate::gui::ui::AkaiVisualizer;
use crate::states::settings_data::SettingsData;
use log::{debug, warn};

impl AkaiVisualizer {
    fn save_settings(&mut self, settings_data: SettingsData) {
        if let Err(e) = settings_data.write_to_config(None) {
            warn!("Failed to save settings {e:?}");
        } else {
            self.settings_data = settings_data;
        }
    }

    pub fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Settings");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Music folder:");
                ui.text_edit_singleline(&mut self.settings_data.music_folder);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.settings_data.music_folder = path.display().to_string();
                    }
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Ambience folder:");
                ui.text_edit_singleline(&mut self.settings_data.ambience_folder);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.settings_data.ambience_folder = path.display().to_string();
                    }
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Sound effect folder:");
                ui.text_edit_singleline(&mut self.settings_data.sound_effect_folder);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.settings_data.sound_effect_folder = path.display().to_string();
                    }
                }
            });

            ui.add_space(10.0);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if ui.button("Save Settings").clicked() {
                    debug!("Saving settings data");
                    self.save_settings(self.settings_data.clone());
                    if let Ok(g_d) = self.gui_data.lock()
                        && let Ok(mut s) = g_d.data.settings_data.lock()
                    {
                        s.copy_data(&self.settings_data);
                    }
                    self.send_command_to_watchdog(Command::Refresh {});
                }
            });
        });
    }
}
