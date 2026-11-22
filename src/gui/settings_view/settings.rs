use crate::audio::output::OutputKind;
use crate::gui::comms::command::CommsCommand;
use crate::gui::ui::AkaiVisualizer;
use crate::states::settings_data::SettingsData;
use rfd::FileDialog;
use std::env;
use std::path::PathBuf;
use tracing::{debug, warn};

fn music_dir() -> anyhow::Result<PathBuf> {
    let output = std::process::Command::new("xdg-user-dir")
        .arg("MUSIC")
        .output()?;

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(
        if path.is_empty()
            && let Some(home) = dirs::home_dir()
        {
            home.join("Music")
        } else {
            PathBuf::from(path)
        },
    )
}

fn get_default_file_dialog() -> FileDialog {
    music_dir().map_or_else(
        |_| FileDialog::new(),
        |path| FileDialog::new().set_directory(path),
    )
}

const fn output_kind_label(kind: OutputKind) -> &'static str {
    match kind {
        OutputKind::Local => "PC audio (local sources)",
        OutputKind::Discord => "Discord (voice channel)",
    }
}

fn snowflake_edit(ui: &mut egui::Ui, value: &mut String, hint: &str) {
    let response = ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(hint)
            .desired_width(220.0),
    );
    if response.changed() {
        value.retain(|c| c.is_ascii_digit());
    }
}

impl AkaiVisualizer {
    fn save_settings(&mut self, settings_data: SettingsData) {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yml".to_string());
        if let Err(e) = settings_data.write_to_config(&config_path) {
            warn!("Failed to save settings {e:?}");
        } else {
            self.settings_data = settings_data;
        }
    }

    pub fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        #[cfg(feature = "profiling")]
        puffin_egui::puffin::profile_function!();
        ui.vertical(|ui| {
            ui.heading("Settings");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Music folder:");
                ui.text_edit_singleline(&mut self.settings_data.music_folder);
                if ui.button("Browse...").clicked()
                    && let Some(path) = get_default_file_dialog().pick_folder()
                {
                    self.settings_data.music_folder = path.display().to_string();
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Ambience folder:");
                ui.text_edit_singleline(&mut self.settings_data.ambience_folder);
                if ui.button("Browse...").clicked()
                    && let Some(path) = get_default_file_dialog().pick_folder()
                {
                    self.settings_data.ambience_folder = path.display().to_string();
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Sound effect folder:");
                ui.text_edit_singleline(&mut self.settings_data.sound_effect_folder);
                if ui.button("Browse...").clicked()
                    && let Some(path) = get_default_file_dialog().pick_folder()
                {
                    self.settings_data.sound_effect_folder = path.display().to_string();
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label(format!(
                    "App refresh rate (detected:{} Hz):",
                    self.settings_data.detected_display_hz
                ));
                let mut current_hz = self.settings_data.repaint_display_hz;
                if ui
                    .add(egui::Slider::new(&mut current_hz, 1.0..=500.).suffix(" Hz"))
                    .changed()
                {
                    self.settings_data.repaint_display_hz = current_hz;
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);
            ui.strong("Audio output");
            ui.add_space(5.0);

            self.render_audio_output_settings(ui);

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
                    self.settings_data.apply_audio();
                    self.send_command_to_watchdog(CommsCommand::Refresh {});
                }
            });
        });
    }
    fn render_audio_output_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Audio output:");
            egui::ComboBox::from_id_salt("audio_output_combo")
                .selected_text(output_kind_label(self.settings_data.audio_output))
                .show_ui(ui, |ui| {
                    for kind in [OutputKind::Local, OutputKind::Discord] {
                        ui.selectable_value(
                            &mut self.settings_data.audio_output,
                            kind,
                            output_kind_label(kind),
                        );
                    }
                });
        });

        ui.add_space(5.0);

        // Discord options only matter when Discord is the selected output.
        let discord_selected = self.settings_data.audio_output == OutputKind::Discord;
        ui.add_enabled_ui(discord_selected, |ui| {
            ui.horizontal(|ui| {
                ui.label("Discord bot token:");

                let show_id = ui.id().with("show_discord_token");
                let mut show = ui.data_mut(|d| d.get_temp::<bool>(show_id).unwrap_or(false));

                ui.add(
                    egui::TextEdit::singleline(&mut self.settings_data.discord_bot_token)
                        .password(!show)
                        .hint_text("Bot token")
                        .desired_width(320.0),
                );
                if ui.button(if show { "Hide" } else { "Show" }).clicked() {
                    show = !show;
                }
                ui.data_mut(|d| d.insert_temp(show_id, show));
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Discord server ID:");
                snowflake_edit(ui, &mut self.settings_data.discord_server_id, "Guild ID");
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Discord voice channel ID:");
                snowflake_edit(
                    ui,
                    &mut self.settings_data.discord_voice_channel_id,
                    "Voice channel ID",
                );
            });

            if discord_selected
                && (self.settings_data.discord_bot_token.trim().is_empty()
                    || self.settings_data.discord_server_id.is_empty()
                    || self.settings_data.discord_voice_channel_id.is_empty())
            {
                ui.add_space(3.0);
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    "Token, server ID and voice channel ID are all required for Discord output.",
                );
            }
        });
    }
}
