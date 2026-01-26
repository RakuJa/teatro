use crate::gui::comms::command::CommsCommand;
use crate::gui::ui::AkaiVisualizer;
use crate::states::knob_value_update::KnobValueUpdate;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{Color32, FontFamily, FontId};
use egui::{Frame, RichText, ScrollArea};

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
            && let Some(ref playlist) = gui_data.data.current_playlist
        {
            let current_track_index = playlist.current_track;
            if let Some(track) = playlist.tracks.get(current_track_index as usize) {
                let new_elapsed = gui_data
                    .audio_player_states
                    .local_elapsed
                    .saturating_add(delta_time_ms);
                if track.track_length > 0 {
                    gui_data.audio_player_states.local_elapsed =
                        new_elapsed.min(track.track_length * 1000);
                } else {
                    gui_data.audio_player_states.local_elapsed = new_elapsed;
                }
            } else {
                gui_data.audio_player_states.local_elapsed = 0;
            }
        }
    }

    fn draw_information_list(&mut self, ui: &mut egui::Ui, scale: f32) {
        Frame::new()
            .fill(Color32::from_rgb(25, 25, 30))
            .stroke(egui::Stroke::new(1.5, Color32::from_rgb(70, 70, 85)))
            .corner_radius(12.0)
            .inner_margin(16.0)
            .shadow(egui::epaint::Shadow {
                offset: [0, 2],
                blur: 8,
                spread: 0,
                color: Color32::from_black_alpha(40),
            })
            .show(ui, |ui| {
                ui.set_width(280.0 * scale);

                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Information Tracker")
                            .size(18.0 * scale)
                            .strong()
                            .color(Color32::from_rgb(230, 230, 240)),
                    );
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(8.0);

                let mut move_up = None;
                let mut move_down = None;
                let mut delete_index = None;

                ScrollArea::vertical()
                    .max_height(400.0 * scale)
                    .show(ui, |ui| {
                        for i in 0..self.info_panel_data.initial_n_of_info {
                            let is_editing = self.info_panel_data.editing_index == Some(i);

                            Frame::new()
                                .fill(if is_editing {
                                    Color32::from_rgb(50, 50, 60)
                                } else {
                                    Color32::from_rgb(40, 40, 50)
                                })
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    if is_editing {
                                        Color32::from_rgb(100, 150, 255)
                                    } else {
                                        Color32::from_rgb(60, 60, 75)
                                    },
                                ))
                                .corner_radius(8.0)
                                .inner_margin(egui::vec2(12.0, 10.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.spacing_mut().item_spacing.y = 4.0;

                                            let up_enabled = i > 0;

                                            // Up button with custom arrow
                                            let up_button_size =
                                                egui::vec2(28.0 * scale, 20.0 * scale);
                                            let (up_rect, up_response) = ui.allocate_exact_size(
                                                up_button_size,
                                                egui::Sense::click(),
                                            );

                                            if ui.is_rect_visible(up_rect) {
                                                let _ = ui.style().interact(&up_response);

                                                // Draw button background
                                                ui.painter().rect_filled(
                                                    up_rect,
                                                    4.0,
                                                    if up_enabled {
                                                        if up_response.hovered() {
                                                            Color32::from_rgb(70, 90, 140)
                                                        } else {
                                                            Color32::from_rgb(60, 80, 120)
                                                        }
                                                    } else {
                                                        Color32::from_rgb(45, 45, 55)
                                                    },
                                                );

                                                ui.painter().rect_stroke(
                                                    up_rect,
                                                    4.0,
                                                    egui::Stroke::new(
                                                        1.0,
                                                        if up_enabled {
                                                            Color32::from_rgb(100, 140, 220)
                                                        } else {
                                                            Color32::from_rgb(60, 60, 70)
                                                        },
                                                    ),
                                                    egui::epaint::StrokeKind::Outside,
                                                );

                                                // Draw up arrow (triangle)
                                                let arrow_color = if up_enabled {
                                                    Color32::from_rgb(180, 200, 255)
                                                } else {
                                                    Color32::from_rgb(80, 80, 90)
                                                };

                                                let center = up_rect.center();
                                                let arrow_size = 6.0 * scale;
                                                let top =
                                                    center + egui::vec2(0.0, -arrow_size / 2.0);
                                                let bottom_left = center
                                                    + egui::vec2(
                                                        -arrow_size / 1.5,
                                                        arrow_size / 2.0,
                                                    );
                                                let bottom_right = center
                                                    + egui::vec2(
                                                        arrow_size / 1.5,
                                                        arrow_size / 2.0,
                                                    );

                                                ui.painter().add(egui::Shape::convex_polygon(
                                                    vec![top, bottom_left, bottom_right],
                                                    arrow_color,
                                                    egui::Stroke::NONE,
                                                ));
                                            }

                                            if up_response.clicked() && up_enabled {
                                                move_up = Some(i);
                                            }

                                            let down_enabled =
                                                i < self.info_panel_data.initial_n_of_info - 1;

                                            // Down button with custom arrow
                                            let down_button_size =
                                                egui::vec2(28.0 * scale, 20.0 * scale);
                                            let (down_rect, down_response) = ui
                                                .allocate_exact_size(
                                                    down_button_size,
                                                    egui::Sense::click(),
                                                );

                                            if ui.is_rect_visible(down_rect) {
                                                let _ = ui.style().interact(&down_response);

                                                // Draw button background
                                                ui.painter().rect_filled(
                                                    down_rect,
                                                    4.0,
                                                    if down_enabled {
                                                        if down_response.hovered() {
                                                            Color32::from_rgb(70, 90, 140)
                                                        } else {
                                                            Color32::from_rgb(60, 80, 120)
                                                        }
                                                    } else {
                                                        Color32::from_rgb(45, 45, 55)
                                                    },
                                                );

                                                ui.painter().rect_stroke(
                                                    down_rect,
                                                    4.0,
                                                    egui::Stroke::new(
                                                        1.0,
                                                        if down_enabled {
                                                            Color32::from_rgb(100, 140, 220)
                                                        } else {
                                                            Color32::from_rgb(60, 60, 70)
                                                        },
                                                    ),
                                                    egui::epaint::StrokeKind::Outside,
                                                );

                                                // Draw down arrow (triangle)
                                                let arrow_color = if down_enabled {
                                                    Color32::from_rgb(180, 200, 255)
                                                } else {
                                                    Color32::from_rgb(80, 80, 90)
                                                };

                                                let center = down_rect.center();
                                                let arrow_size = 6.0 * scale;
                                                let bottom =
                                                    center + egui::vec2(0.0, arrow_size / 2.0);
                                                let top_left = center
                                                    + egui::vec2(
                                                        -arrow_size / 1.5,
                                                        -arrow_size / 2.0,
                                                    );
                                                let top_right = center
                                                    + egui::vec2(
                                                        arrow_size / 1.5,
                                                        -arrow_size / 2.0,
                                                    );

                                                ui.painter().add(egui::Shape::convex_polygon(
                                                    vec![bottom, top_left, top_right],
                                                    arrow_color,
                                                    egui::Stroke::NONE,
                                                ));
                                            }

                                            if down_response.clicked() && down_enabled {
                                                move_down = Some(i);
                                            }
                                        });

                                        ui.add_space(10.0);

                                        // Vertical divider
                                        let divider_rect = Rect::from_min_size(
                                            ui.cursor().min,
                                            egui::vec2(1.5, ui.available_height()),
                                        );
                                        ui.painter().rect_filled(
                                            divider_rect,
                                            1.0,
                                            Color32::from_rgb(80, 80, 95),
                                        );

                                        ui.add_space(10.0);

                                        // Text content
                                        ui.vertical(|ui| {
                                            if is_editing {
                                                let edit_response = ui.add(
                                                    egui::TextEdit::singleline(
                                                        &mut self.info_panel_data.combattant_lists
                                                            [i],
                                                    )
                                                    .desired_width(f32::INFINITY)
                                                    .font(egui::TextStyle::Body)
                                                    .text_color(Color32::from_rgb(240, 240, 250)),
                                                );

                                                if edit_response.lost_focus()
                                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                                {
                                                    self.info_panel_data.editing_index = None;
                                                }

                                                if edit_response.lost_focus()
                                                    && ui
                                                        .input(|i| i.key_pressed(egui::Key::Escape))
                                                {
                                                    self.info_panel_data.editing_index = None;
                                                }

                                                edit_response.request_focus();
                                            } else {
                                                let label_response = ui.add(
                                                    egui::Label::new(
                                                        RichText::new(
                                                            &self.info_panel_data.combattant_lists
                                                                [i],
                                                        )
                                                        .color(Color32::from_rgb(220, 220, 235))
                                                        .size(14.0 * scale),
                                                    )
                                                    .wrap()
                                                    .sense(egui::Sense::click()),
                                                );

                                                if label_response.double_clicked() {
                                                    self.info_panel_data.editing_index = Some(i);
                                                }

                                                if label_response.hovered() {
                                                    ui.ctx().set_cursor_icon(
                                                        egui::CursorIcon::PointingHand,
                                                    );
                                                }

                                                label_response
                                                    .on_hover_text("Double-click to edit");
                                            }
                                        });

                                        ui.add_space(10.0);

                                        ui.vertical(|ui| {
                                            ui.add_space(2.0);

                                            let delete_button_size =
                                                egui::vec2(24.0 * scale, 24.0 * scale);
                                            let (delete_rect, delete_response) = ui
                                                .allocate_exact_size(
                                                    delete_button_size,
                                                    egui::Sense::click(),
                                                );

                                            if ui.is_rect_visible(delete_rect) {
                                                ui.painter().rect_filled(
                                                    delete_rect,
                                                    4.0,
                                                    if delete_response.hovered() {
                                                        Color32::from_rgb(180, 60, 60)
                                                    } else {
                                                        Color32::from_rgb(120, 50, 50)
                                                    },
                                                );

                                                ui.painter().rect_stroke(
                                                    delete_rect,
                                                    4.0,
                                                    egui::Stroke::new(
                                                        1.0,
                                                        Color32::from_rgb(200, 80, 80),
                                                    ),
                                                    egui::epaint::StrokeKind::Outside,
                                                );

                                                let center = delete_rect.center();
                                                let x_size = 8.0 * scale;
                                                let x_color = Color32::from_rgb(255, 200, 200);

                                                ui.painter().line_segment(
                                                    [
                                                        center
                                                            + egui::vec2(
                                                                -x_size / 2.0,
                                                                -x_size / 2.0,
                                                            ),
                                                        center
                                                            + egui::vec2(
                                                                x_size / 2.0,
                                                                x_size / 2.0,
                                                            ),
                                                    ],
                                                    egui::Stroke::new(2.0 * scale, x_color),
                                                );

                                                ui.painter().line_segment(
                                                    [
                                                        center
                                                            + egui::vec2(
                                                                x_size / 2.0,
                                                                -x_size / 2.0,
                                                            ),
                                                        center
                                                            + egui::vec2(
                                                                -x_size / 2.0,
                                                                x_size / 2.0,
                                                            ),
                                                    ],
                                                    egui::Stroke::new(2.0 * scale, x_color),
                                                );
                                            }

                                            if delete_response.clicked() {
                                                delete_index = Some(i);
                                            }

                                            delete_response.on_hover_text("Delete item");
                                        });
                                    });
                                });

                            ui.add_space(8.0);
                        }
                    });

                ui.add_space(12.0);

                // Add new item button
                let add_button = ui.add(
                    egui::Button::new(
                        RichText::new("+ Add New Item")
                            .size(14.0 * scale)
                            .color(Color32::from_rgb(200, 220, 255)),
                    )
                    .fill(Color32::from_rgb(50, 80, 120))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(100, 140, 220)))
                    .corner_radius(6.0)
                    .min_size(egui::vec2(ui.available_width(), 32.0 * scale)),
                );

                if add_button.clicked() {
                    self.info_panel_data
                        .combattant_lists
                        .push(String::from("New Item"));
                    self.info_panel_data.initial_n_of_info += 1;
                    self.info_panel_data.editing_index =
                        Some(self.info_panel_data.initial_n_of_info - 1);
                }

                // Handle deletion after the loop
                if let Some(idx) = delete_index {
                    if self.info_panel_data.initial_n_of_info > 0 {
                        self.info_panel_data.combattant_lists.remove(idx);
                        self.info_panel_data.initial_n_of_info -= 1;
                        if let Some(editing) = self.info_panel_data.editing_index {
                            if editing == idx {
                                self.info_panel_data.editing_index = None;
                            } else if editing > idx {
                                self.info_panel_data.editing_index = Some(editing - 1);
                            }
                        }
                    }
                }

                // Handle movement after the loop
                if let Some(idx) = move_up {
                    if idx > 0 {
                        self.info_panel_data.combattant_lists.swap(idx, idx - 1);
                    }
                }

                if let Some(idx) = move_down {
                    if idx < self.info_panel_data.initial_n_of_info - 1 {
                        self.info_panel_data.combattant_lists.swap(idx, idx + 1);
                    }
                }
            });
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
