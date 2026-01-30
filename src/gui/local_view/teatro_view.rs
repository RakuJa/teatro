use crate::gui::comms::command::CommsCommand;
use crate::gui::ui::AkaiVisualizer;
use crate::states::knob_value_update::KnobValueUpdate;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{Color32, FontFamily, FontId};
use egui::RichText;

impl AkaiVisualizer {
    pub(crate) fn render_visualizer_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(
            RichText::new("Code By RakuJa")
                .color(Color32::from_rgb(102, 0, 51))
                .font(FontId {
                    size: 15.0,
                    family: FontFamily::Name("Pixelify".into()),
                }),
        );
        ui.heading(
            RichText::new("Teatro - TTRPG Helper")
                .color(Color32::PURPLE)
                .font(FontId {
                    size: 25.0,
                    family: FontFamily::Name("Pixelify".into()),
                }),
        );
        ui.add_space(10.0);

        let available_size = ui.available_size();
        let base_width = 1300.0;
        let base_height = 700.0;
        let scale = (available_size.x / base_width)
            .min(available_size.y / base_height)
            .min(2.0);

        ui.horizontal(|ui| {
            let controller_size = Vec2::new(900.0 * scale, base_height * scale);
            let (response, painter) = ui.allocate_painter(controller_size, egui::Sense::hover());

            let rect = response.rect;
            painter.rect_filled(rect, 5.0 * scale, Color32::from_rgb(30, 30, 35));

            self.draw_pads(ui, rect, scale);
            self.draw_knobs(ui, rect, scale);
            self.draw_keyboard(ui, rect, scale);
            self.draw_audio_player(ui, rect, scale);

            ui.add_space(82.0);

            ui.vertical(|ui| {
                self.draw_information_list(ui, scale);
            })
        });

        ui.add_space(5.0);
        ui.label(
            RichText::new(format!(
                "Music folder: {}",
                self.gui_data
                    .lock()
                    .map(|x| x
                        .data
                        .settings_data
                        .lock()
                        .map_or_else(|_| "music".to_string(), |x| x.music_folder.clone()))
                    .unwrap_or_default()
            ))
            .color(Color32::from_rgb(180, 100, 220))
            .font(FontId {
                size: 14.0,
                family: FontFamily::Name("Pixelify".into()),
            }),
        );
    }
    pub(crate) fn update_local_progress(&self, delta_time_ms: u64) {
        if let Ok(mut gui_data) = self.gui_data.lock()
            && gui_data.player_info.status.is_music_playable()
            && let Some(ref playlist) = gui_data.data.current_playlist
        {
            let current_track_index = playlist.current_track;
            if let Some(track) = playlist.tracks.get(current_track_index as usize) {
                let new_elapsed = gui_data
                    .player_info
                    .local_elapsed
                    .saturating_add(delta_time_ms);
                if track.track_length > 0 {
                    gui_data.player_info.local_elapsed = new_elapsed.min(track.track_length * 1000);
                } else {
                    gui_data.player_info.local_elapsed = new_elapsed;
                }
            } else {
                gui_data.player_info.local_elapsed = 0;
            }
        }
    }

    fn draw_pads(&self, ui: &mut egui::Ui, rect: Rect, scale: f32) {
        let pad_size = 50.0 * scale;
        let pad_spacing = 10.0 * scale;
        let start_x = 20.0f32.mul_add(scale, rect.min.x);
        let start_y = 10.0f32.mul_add(scale, rect.min.y);

        for row in 0..5 {
            for col in 0..8 {
                let idx = (4 - row) * 8 + col;
                let x = f32::from(col).mul_add(pad_size + pad_spacing, start_x);
                let y = f32::from(row).mul_add(pad_size + pad_spacing, start_y);

                let pad_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(pad_size, pad_size));

                let pad_response = ui.allocate_rect(pad_rect, egui::Sense::click());
                if pad_response.clicked() {
                    self.send_command_to_backend(CommsCommand::PadPressed { key: idx });
                }

                // Enhanced pad colors with glow effect
                let (base_color, glow_color) = if let Ok(gui_data) = self.gui_data.lock()
                    && gui_data.data.last_pad_pressed.is_some_and(|x| x == idx)
                {
                    (
                        Color32::from_rgb(140, 60, 180),
                        Color32::from_rgba_premultiplied(180, 100, 220, 80),
                    )
                } else {
                    (
                        Color32::from_rgb(80, 180, 100),
                        Color32::from_rgba_premultiplied(100, 200, 120, 60),
                    )
                };

                let glow_rect = pad_rect.expand(2.0 * scale);
                ui.painter().rect_filled(glow_rect, 4.0 * scale, glow_color);

                ui.painter().rect_filled(pad_rect, 4.0 * scale, base_color);
                ui.painter().rect_stroke(
                    pad_rect,
                    4.0 * scale,
                    egui::Stroke::new(2.0 * scale, Color32::from_rgb(20, 20, 25)),
                    egui::StrokeKind::Outside,
                );

                ui.painter().text(
                    Pos2::new(6.0f32.mul_add(scale, x), 6.0f32.mul_add(scale, y)),
                    egui::Align2::LEFT_TOP,
                    format!("{idx}"),
                    FontId::proportional(11.0 * scale),
                    Color32::from_rgba_premultiplied(255, 255, 255, 180),
                );

                if let Ok(gui_data) = self.gui_data.lock()
                    && let Some(label) = gui_data.data.pad_labels.get(idx as usize)
                    && !label.is_empty()
                {
                    ui.painter().text(
                        pad_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        FontId::proportional(12.0 * scale),
                        Color32::WHITE,
                    );
                }
            }
        }
    }

    fn draw_knobs(&self, ui: &mut egui::Ui, rect: Rect, scale: f32) {
        let knob_radius = 22.0 * scale;
        let knob_spacing = 60.0 * scale;
        let start_x = 40.0f32.mul_add(scale, rect.min.x);
        let start_y = 340.0f32.mul_add(scale, rect.min.y);

        for i in 0u8..8 {
            let x = f32::from(i).mul_add(knob_spacing, start_x);
            let y = start_y;
            let center = Pos2::new(x, y);

            let current_value = self.gui_data.lock().map_or(0.5, |gui_data| {
                *gui_data.data.knob_values.get(&(i + 1)).unwrap_or(&0.5f32)
            });

            ui.painter().circle_filled(
                center,
                3.0f32.mul_add(scale, knob_radius),
                Color32::from_rgba_premultiplied(100, 100, 120, 40),
            );

            ui.painter()
                .circle_filled(center, knob_radius, Color32::from_rgb(50, 50, 60));

            ui.painter()
                .circle_filled(center, knob_radius * 0.85, Color32::from_rgb(65, 65, 75));

            ui.painter().circle_stroke(
                center,
                knob_radius,
                egui::Stroke::new(2.5 * scale, Color32::from_rgb(90, 90, 100)),
            );

            let angle = -2.4f32 + current_value * 4.8;
            let indicator_len = knob_radius * 0.7;
            let end = Pos2::new(
                angle.cos().mul_add(indicator_len, x),
                angle.sin().mul_add(indicator_len, y),
            );

            ui.painter().line_segment(
                [center, Pos2::new(end.x + scale, end.y + scale)],
                egui::Stroke::new(4.0 * scale, Color32::from_rgba_premultiplied(0, 0, 0, 100)),
            );

            ui.painter().line_segment(
                [center, end],
                egui::Stroke::new(4.0 * scale, Color32::from_rgb(220, 160, 255)),
            );

            ui.painter().text(
                Pos2::new(x, 18.0f32.mul_add(scale, y + knob_radius)),
                egui::Align2::CENTER_TOP,
                format!("K{}", i + 1),
                FontId::proportional(11.0 * scale),
                Color32::from_rgb(200, 200, 210),
            );

            let display_pos = Pos2::new(x, 32.0f32.mul_add(scale, y + knob_radius));
            ui.painter().text(
                display_pos,
                egui::Align2::CENTER_TOP,
                format!("{:.0}%", current_value * 100.0),
                FontId::proportional(20.0 * scale),
                Color32::from_rgb(160, 160, 170),
            );

            let button_y = 68.0f32.mul_add(scale, y + knob_radius);
            let button_size = Vec2::new(20.0 * scale, 16.0 * scale);
            let button_spacing = 4.0 * scale;

            let dec_rect = Rect::from_center_size(
                Pos2::new(x - (button_size.x / 2.0 + button_spacing / 2.0), button_y),
                button_size,
            );

            let dec_response = ui.put(dec_rect, egui::Button::new("-").small());
            if dec_response.clicked() {
                self.send_command_to_backend(CommsCommand::KnobPercentageChanged {
                    knob: i + 1,
                    value: KnobValueUpdate::Decrement,
                });
            }

            let inc_rect = Rect::from_center_size(
                Pos2::new(x + (button_size.x / 2.0 + button_spacing / 2.0), button_y),
                button_size,
            );

            let inc_response = ui.put(inc_rect, egui::Button::new("+").small());
            if inc_response.clicked() {
                self.send_command_to_backend(CommsCommand::KnobPercentageChanged {
                    knob: i + 1,
                    value: KnobValueUpdate::Increment,
                });
            }
        }
    }

    fn draw_keyboard(&self, ui: &mut egui::Ui, rect: Rect, scale: f32) {
        let white_key_width = 35.0 * scale;
        let white_key_height = 100.0 * scale;
        let black_key_width = 22.0 * scale;
        let black_key_height = 65.0 * scale;
        let start_x = 20.0f32.mul_add(scale, rect.min.x);
        let start_y = 450.0f32.mul_add(scale, rect.min.y);

        let pattern = [
            true, false, true, false, true, true, false, true, false, true, false, true,
        ];

        let mut white_idx = 0;
        for i in 0..25u8 {
            let is_white = pattern[i as usize % 12];
            if is_white {
                let x = (white_idx as f32).mul_add(white_key_width, start_x);
                let key_rect = Rect::from_min_size(
                    Pos2::new(x, start_y),
                    Vec2::new(white_key_width, white_key_height),
                );

                let key_response = ui.allocate_rect(key_rect, egui::Sense::click());

                if key_response.clicked() {
                    self.send_command_to_backend(CommsCommand::WhiteKeyPressed { key: i + 1 });
                }

                ui.painter()
                    .rect_filled(key_rect, 3.0 * scale, Color32::from_rgb(250, 250, 250));

                let highlight_rect = Rect::from_min_size(
                    key_rect.min,
                    Vec2::new(key_rect.width(), key_rect.height() * 0.15),
                );
                ui.painter().rect_filled(
                    highlight_rect,
                    3.0 * scale,
                    Color32::from_rgba_premultiplied(255, 255, 255, 150),
                );

                ui.painter().rect_stroke(
                    key_rect,
                    3.0 * scale,
                    egui::Stroke::new(2.0 * scale, Color32::from_rgb(40, 40, 45)),
                    egui::StrokeKind::Outside,
                );
                white_idx += 1;
            }
        }

        white_idx = 0;
        for i in 0..25_u8 {
            let is_white = pattern[i as usize % 12];
            if is_white {
                white_idx += 1;
            } else {
                let x =
                    (white_idx as f32).mul_add(white_key_width, start_x) - black_key_width / 2.0;
                let key_rect = Rect::from_min_size(
                    Pos2::new(x, start_y),
                    Vec2::new(black_key_width, black_key_height),
                );

                let key_response = ui.allocate_rect(key_rect, egui::Sense::click());

                if key_response.clicked() {
                    self.send_command_to_backend(CommsCommand::BlackKeyPressed { key: i + 1 });
                }

                ui.painter()
                    .rect_filled(key_rect, 3.0 * scale, Color32::from_rgb(25, 25, 30));

                let highlight_rect = Rect::from_min_size(
                    key_rect.min,
                    Vec2::new(key_rect.width(), key_rect.height() * 0.2),
                );
                ui.painter().rect_filled(
                    highlight_rect,
                    3.0 * scale,
                    Color32::from_rgba_premultiplied(60, 60, 70, 100),
                );

                ui.painter().rect_stroke(
                    key_rect,
                    3.0 * scale,
                    egui::Stroke::new(1.5 * scale, Color32::BLACK),
                    egui::StrokeKind::Outside,
                );
            }
        }
    }
}
