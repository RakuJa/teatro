use crate::comms::command::{Command, Device};
use crate::states::visualizer::AkaiData;
use eframe::egui;
use egui::{Color32, FontFamily, FontId, Frame, Pos2, Rect, RichText, ScrollArea, Vec2};
use egui_font_loader::{LoaderFontData, load_fonts};
use flume::Sender;
use log::warn;
use std::time::{Duration, Instant};

pub struct AkaiVisualizer {
    pub data: AkaiData,
    pub tx_command: Sender<Command>,
    last_refresh: Instant,
    last_local_update: Instant,
    refresh_interval: Duration,
    pub local_elapsed: u64,

    combattant_lists: Vec<String>,
    initial_n_of_info: usize,
    editing_index: Option<usize>,
    dragging_index: Option<usize>,
}

impl AkaiVisualizer {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        data: AkaiData,
        tx_command: Sender<Command>,
        font_folder: &str,
        initial_n_of_info: usize,
    ) -> Self {
        let fonts = vec![
            LoaderFontData {
                name: "GoodTimesRg".into(),
                path: format!("{font_folder}/Good-times-rg.ttf"),
            },
            LoaderFontData {
                name: "Pixelify".into(),
                path: format!("{font_folder}/PixelifySans-VariableFont_wght.ttf"),
            },
        ];
        load_fonts(&cc.egui_ctx, fonts).expect("Font should be readable.");
        Self {
            data,
            tx_command,
            last_refresh: Instant::now(),
            last_local_update: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            local_elapsed: 0,

            combattant_lists: (0..initial_n_of_info)
                .map(|i| format!("Combattant {i} | Initiative: 0"))
                .collect(),
            initial_n_of_info,
            editing_index: None,
            dragging_index: None,
        }
    }

    fn update_local_progress(&mut self, delta_time_ms: u64) {
        if let Some(ref playlist) = self.data.current_playlist {
            let current_track_index = playlist.current_track;
            if let Some(track) = playlist.tracks.get(current_track_index as usize) {
                self.local_elapsed = self.local_elapsed.saturating_add(delta_time_ms);
                if track.track_length > 0 {
                    self.local_elapsed = self.local_elapsed.min(track.track_length * 1000);
                }
            } else {
                self.local_elapsed = 0;
            }
        } else {
            self.local_elapsed = 0;
        }
    }

    fn format_time(milliseconds: u64) -> String {
        let total_seconds = milliseconds / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{minutes:02}:{seconds:02}")
    }
}

impl eframe::App for AkaiVisualizer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();

        let delta_time = now.duration_since(self.last_local_update).as_millis() as u64;
        self.last_local_update = now;
        if now.duration_since(self.last_refresh) >= self.refresh_interval {
            if let Err(e) = self.tx_command.send(Command::Refresh {
                device: Device::ToBackend,
            }) {
                warn!("Failed to send refresh command: {e}");
            }
            self.last_refresh = now;
        }
        self.update_local_progress(delta_time);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(
                RichText::new("Code By RakuJa")
                    .color(Color32::from_rgb(102, 0, 51))
                    .font(FontId {
                        size: 15.0,
                        family: FontFamily::Name("Pixelify".into()),
                    }),
            );
            ui.heading(
                RichText::new("Teatro - Akai APC Key 25 Controller Visualizer")
                    .color(Color32::PURPLE)
                    .font(FontId {
                        size: 25.0,
                        family: FontFamily::Name("Pixelify".into()),
                    }),
            );
            ui.add_space(10.0);

            let available_size = ui.available_size();
            let base_width = 1150.0;
            let base_height = 650.0;
            let scale = (available_size.x / base_width)
                .min(available_size.y / base_height)
                .min(2.0);

            ui.horizontal(|ui| {
                let controller_size = Vec2::new(900.0 * scale, base_height * scale);
                let (response, painter) =
                    ui.allocate_painter(controller_size, egui::Sense::hover());

                let rect = response.rect;
                painter.rect_filled(rect, 5.0 * scale, Color32::from_rgb(30, 30, 35));

                self.draw_pads(&painter, rect, scale);
                self.draw_knobs(&painter, rect, scale);
                Self::draw_keyboard(&painter, rect, scale);
                self.draw_audio_player(&painter, rect, scale);

                ui.add_space(10.0);

                ui.vertical(|ui| {
                    self.draw_information_list(ui, scale);
                })
            });

            ui.add_space(5.0);
            ui.label(
                RichText::new(format!("Music folder: {}", self.data.music_folder))
                    .color(Color32::from_rgb(180, 100, 220))
                    .font(FontId {
                        size: 14.0,
                        family: FontFamily::Name("Pixelify".into()),
                    }),
            );
        });
    }
}

impl AkaiVisualizer {
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
                    .max_height(520.0 * scale)
                    .show(ui, |ui| {
                        for i in 0..self.initial_n_of_info {
                            let is_editing = self.editing_index == Some(i);

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

                                            let down_enabled = i < self.initial_n_of_info - 1;

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
                                                        &mut self.combattant_lists[i],
                                                    )
                                                    .desired_width(f32::INFINITY)
                                                    .font(egui::TextStyle::Body)
                                                    .text_color(Color32::from_rgb(240, 240, 250)),
                                                );

                                                if edit_response.lost_focus()
                                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                                {
                                                    self.editing_index = None;
                                                }

                                                if edit_response.lost_focus()
                                                    && ui
                                                        .input(|i| i.key_pressed(egui::Key::Escape))
                                                {
                                                    self.editing_index = None;
                                                }

                                                edit_response.request_focus();
                                            } else {
                                                let label_response = ui.add(
                                                    egui::Label::new(
                                                        RichText::new(&self.combattant_lists[i])
                                                            .color(Color32::from_rgb(220, 220, 235))
                                                            .size(14.0 * scale),
                                                    )
                                                    .wrap()
                                                    .sense(egui::Sense::click()),
                                                );

                                                if label_response.double_clicked() {
                                                    self.editing_index = Some(i);
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
                    self.combattant_lists.push(String::from("New Item"));
                    self.initial_n_of_info += 1;
                    self.editing_index = Some(self.initial_n_of_info - 1);
                }

                // Handle deletion after the loop
                if let Some(idx) = delete_index {
                    if self.initial_n_of_info > 0 {
                        self.combattant_lists.remove(idx);
                        self.initial_n_of_info -= 1;
                        if let Some(editing) = self.editing_index {
                            if editing == idx {
                                self.editing_index = None;
                            } else if editing > idx {
                                self.editing_index = Some(editing - 1);
                            }
                        }
                    }
                }

                // Handle movement after the loop
                if let Some(idx) = move_up {
                    if idx > 0 {
                        self.combattant_lists.swap(idx, idx - 1);
                    }
                }

                if let Some(idx) = move_down {
                    if idx < self.initial_n_of_info - 1 {
                        self.combattant_lists.swap(idx, idx + 1);
                    }
                }
            });
    }
    fn draw_pads(&self, painter: &egui::Painter, rect: Rect, scale: f32) {
        let pad_size = 50.0 * scale;
        let pad_spacing = 10.0 * scale;
        let start_x = 20.0f32.mul_add(scale, rect.min.x);
        let start_y = 30.0f32.mul_add(scale, rect.min.y);

        for row in 0..5 {
            for col in 0..8 {
                let idx = (4 - row) * 8 + col;
                let x = f32::from(col).mul_add(pad_size + pad_spacing, start_x);
                let y = f32::from(row).mul_add(pad_size + pad_spacing, start_y);

                let pad_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(pad_size, pad_size));

                // Enhanced pad colors with glow effect
                let (base_color, glow_color) =
                    if self.data.last_pad_pressed.is_some_and(|x| x == idx) {
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
                painter.rect_filled(glow_rect, 4.0 * scale, glow_color);

                painter.rect_filled(pad_rect, 4.0 * scale, base_color);
                painter.rect_stroke(
                    pad_rect,
                    4.0 * scale,
                    egui::Stroke::new(2.0 * scale, Color32::from_rgb(20, 20, 25)),
                    egui::StrokeKind::Outside,
                );

                painter.text(
                    Pos2::new(6.0f32.mul_add(scale, x), 6.0f32.mul_add(scale, y)),
                    egui::Align2::LEFT_TOP,
                    format!("{idx}"),
                    FontId::proportional(11.0 * scale),
                    Color32::from_rgba_premultiplied(255, 255, 255, 180),
                );

                if let Some(label) = self.data.pad_labels.get(idx as usize)
                    && !label.is_empty()
                {
                    painter.text(
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

    fn draw_knobs(&self, painter: &egui::Painter, rect: Rect, scale: f32) {
        let knob_radius = 22.0 * scale;
        let knob_spacing = 60.0 * scale;
        let start_x = 40.0f32.mul_add(scale, rect.min.x);
        let start_y = 360.0f32.mul_add(scale, rect.min.y);

        for i in 0u8..8 {
            let x = f32::from(i).mul_add(knob_spacing, start_x);
            let y = start_y;
            let center = Pos2::new(x, y);

            painter.circle_filled(
                center,
                3.0f32.mul_add(scale, knob_radius),
                Color32::from_rgba_premultiplied(100, 100, 120, 40),
            );

            painter.circle_filled(center, knob_radius, Color32::from_rgb(50, 50, 60));

            painter.circle_filled(center, knob_radius * 0.85, Color32::from_rgb(65, 65, 75));

            painter.circle_stroke(
                center,
                knob_radius,
                egui::Stroke::new(2.5 * scale, Color32::from_rgb(90, 90, 100)),
            );

            let value = self.data.knob_values.get(&(i + 1)).unwrap_or(&0f32);
            let angle = -2.4f32 + value * 4.8;
            let indicator_len = knob_radius * 0.7;
            let end = Pos2::new(
                angle.cos().mul_add(indicator_len, x),
                angle.sin().mul_add(indicator_len, y),
            );

            painter.line_segment(
                [center, Pos2::new(end.x + scale, end.y + scale)],
                egui::Stroke::new(4.0 * scale, Color32::from_rgba_premultiplied(0, 0, 0, 100)),
            );

            painter.line_segment(
                [center, end],
                egui::Stroke::new(4.0 * scale, Color32::from_rgb(220, 160, 255)),
            );

            painter.text(
                Pos2::new(x, 18.0f32.mul_add(scale, y + knob_radius)),
                egui::Align2::CENTER_TOP,
                format!("K{}", i + 1),
                FontId::proportional(11.0 * scale),
                Color32::from_rgb(200, 200, 210),
            );

            painter.text(
                Pos2::new(x, 32.0f32.mul_add(scale, y + knob_radius)),
                egui::Align2::CENTER_TOP,
                format!("{:.0}%", value * 100.0),
                FontId::proportional(9.0 * scale),
                Color32::from_rgb(160, 160, 170),
            );
        }
    }

    fn draw_keyboard(painter: &egui::Painter, rect: Rect, scale: f32) {
        let white_key_width = 35.0 * scale;
        let white_key_height = 100.0 * scale;
        let black_key_width = 22.0 * scale;
        let black_key_height = 65.0 * scale;
        let start_x = 530.0f32.mul_add(scale, rect.min.x);
        let start_y = 30.0f32.mul_add(scale, rect.min.y);

        let pattern = [
            true, false, true, false, true, true, false, true, false, true, false, true,
        ];

        let mut white_idx = 0;
        for i in 0..25 {
            let is_white = pattern[i % 12];
            if is_white {
                let x = (white_idx as f32).mul_add(white_key_width, start_x);
                let key_rect = Rect::from_min_size(
                    Pos2::new(x, start_y),
                    Vec2::new(white_key_width, white_key_height),
                );

                painter.rect_filled(key_rect, 3.0 * scale, Color32::from_rgb(250, 250, 250));

                let highlight_rect = Rect::from_min_size(
                    key_rect.min,
                    Vec2::new(key_rect.width(), key_rect.height() * 0.15),
                );
                painter.rect_filled(
                    highlight_rect,
                    3.0 * scale,
                    Color32::from_rgba_premultiplied(255, 255, 255, 150),
                );

                painter.rect_stroke(
                    key_rect,
                    3.0 * scale,
                    egui::Stroke::new(2.0 * scale, Color32::from_rgb(40, 40, 45)),
                    egui::StrokeKind::Outside,
                );
                white_idx += 1;
            }
        }

        white_idx = 0;
        for i in 0..25 {
            let is_white = pattern[i % 12];
            if is_white {
                white_idx += 1;
            } else {
                let x =
                    (white_idx as f32).mul_add(white_key_width, start_x) - black_key_width / 2.0;
                let key_rect = Rect::from_min_size(
                    Pos2::new(x, start_y),
                    Vec2::new(black_key_width, black_key_height),
                );

                painter.rect_filled(key_rect, 3.0 * scale, Color32::from_rgb(25, 25, 30));

                let highlight_rect = Rect::from_min_size(
                    key_rect.min,
                    Vec2::new(key_rect.width(), key_rect.height() * 0.2),
                );
                painter.rect_filled(
                    highlight_rect,
                    3.0 * scale,
                    Color32::from_rgba_premultiplied(60, 60, 70, 100),
                );

                painter.rect_stroke(
                    key_rect,
                    3.0 * scale,
                    egui::Stroke::new(1.5 * scale, Color32::BLACK),
                    egui::StrokeKind::Outside,
                );
            }
        }
    }

    fn draw_audio_player(&self, painter: &egui::Painter, rect: Rect, scale: f32) {
        let player_height = 95.0 * scale;
        let margin = 15.0 * scale;

        let player_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + margin, rect.max.y - player_height - margin),
            Vec2::new(rect.width() - margin * 2.0, player_height),
        );

        painter.rect_filled(player_rect, 10.0 * scale, Color32::from_rgb(18, 18, 22));

        let glow_rect1 = player_rect.shrink(1.5 * scale);
        painter.rect_stroke(
            glow_rect1,
            9.0 * scale,
            egui::Stroke::new(
                3.0 * scale,
                Color32::from_rgba_premultiplied(140, 70, 180, 80),
            ),
            egui::StrokeKind::Inside,
        );

        let glow_rect2 = player_rect.shrink(3.0 * scale);
        painter.rect_stroke(
            glow_rect2,
            8.0 * scale,
            egui::Stroke::new(
                1.5 * scale,
                Color32::from_rgba_premultiplied(180, 100, 220, 40),
            ),
            egui::StrokeKind::Inside,
        );

        let (file_path, progress, elapsed_ms, total_ms) =
            if let Some(playlist) = self.data.current_playlist.clone() {
                let track = playlist.tracks[playlist.current_track as usize].clone();
                let track_length = track.track_length;
                let prog = if track_length > 0 {
                    (self.local_elapsed as f32 / ((track_length * 1000) as f32)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (
                    track.file_path,
                    prog,
                    self.local_elapsed,
                    track_length * 1000,
                )
            } else {
                (String::new(), 0.0, 0, 0)
            };

        let content_padding = 14.0 * scale;

        let title = std::path::Path::new(&file_path)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("No Track Playing")
            .to_string();

        painter.text(
            Pos2::new(
                content_padding + player_rect.min.x + scale,
                content_padding + player_rect.min.y + scale,
            ),
            egui::Align2::LEFT_TOP,
            &title,
            FontId {
                size: 17.0 * scale,
                family: FontFamily::Name("Pixelify".into()),
            },
            Color32::from_rgba_premultiplied(0, 0, 0, 150),
        );

        painter.text(
            Pos2::new(
                content_padding + player_rect.min.x,
                content_padding + player_rect.min.y,
            ),
            egui::Align2::LEFT_TOP,
            &title,
            FontId {
                size: 17.0 * scale,
                family: FontFamily::Name("Pixelify".into()),
            },
            Color32::from_rgb(245, 245, 250),
        );

        let time_text = format!(
            "{} / {}",
            Self::format_time(elapsed_ms),
            Self::format_time(total_ms)
        );

        painter.text(
            Pos2::new(
                player_rect.max.x - content_padding,
                content_padding + player_rect.min.y,
            ),
            egui::Align2::RIGHT_TOP,
            &time_text,
            FontId {
                size: 13.0 * scale,
                family: FontFamily::Name("Pixelify".into()),
            },
            Color32::from_rgb(200, 160, 220),
        );

        // Waveform visualization above progress bar
        if progress > 0.0 {
            let wave_height = 22.0 * scale;
            let wave_y = 38.0f32.mul_add(-scale, player_rect.max.y - content_padding);
            let num_bars = 50;
            let bar_width =
                ((player_rect.width() - content_padding * 2.0) / num_bars as f32) * 0.75;
            let wave_x_start = player_rect.min.x + content_padding;

            for i in 0..num_bars {
                let x = wave_x_start
                    + (i as f32 * (player_rect.width() - content_padding * 2.0) / num_bars as f32);

                let height_factor = (i as f32)
                    .mul_add(0.4, progress * 12.0)
                    .sin()
                    .mul_add(0.5, 0.5)
                    .mul_add(0.7, 0.3);
                let bar_height = wave_height * height_factor;

                let opacity = if (i as f32 / num_bars as f32) <= progress {
                    140
                } else {
                    35
                };

                let wave_rect = Rect::from_min_size(
                    Pos2::new(x, wave_y + wave_height - bar_height),
                    Vec2::new(bar_width, bar_height),
                );

                painter.rect_filled(
                    wave_rect,
                    1.5 * scale,
                    Color32::from_rgba_premultiplied(180, 100, 220, opacity),
                );

                if opacity > 100 {
                    let highlight =
                        Rect::from_min_size(wave_rect.min, Vec2::new(bar_width, bar_height * 0.3));
                    painter.rect_filled(
                        highlight,
                        1.5 * scale,
                        Color32::from_rgba_premultiplied(220, 160, 255, opacity / 2),
                    );
                }
            }
        }

        let bar_margin = content_padding;
        let bar_height = 12.0 * scale;
        let bar_y = player_rect.max.y - bar_margin - bar_height;

        let bar_rect = Rect::from_min_size(
            Pos2::new(player_rect.min.x + bar_margin, bar_y),
            Vec2::new(player_rect.width() - bar_margin * 2.0, bar_height),
        );

        painter.rect_filled(bar_rect, 6.0 * scale, Color32::from_rgb(35, 35, 45));

        let shadow_rect =
            Rect::from_min_size(bar_rect.min, Vec2::new(bar_rect.width(), 3.0 * scale));
        painter.rect_filled(
            shadow_rect,
            6.0 * scale,
            Color32::from_rgba_premultiplied(0, 0, 0, 100),
        );

        painter.rect_stroke(
            bar_rect,
            6.0 * scale,
            egui::Stroke::new(1.5 * scale, Color32::from_rgb(55, 55, 65)),
            egui::StrokeKind::Outside,
        );

        if progress > 0.0 {
            let filled_width = bar_rect.width() * progress;
            let filled_rect =
                Rect::from_min_size(bar_rect.min, Vec2::new(filled_width, bar_rect.height()));

            painter.rect_filled(filled_rect, 6.0 * scale, Color32::from_rgb(150, 70, 190));

            let highlight_rect = Rect::from_min_size(
                filled_rect.min,
                Vec2::new(filled_rect.width(), filled_rect.height() * 0.45),
            );
            painter.rect_filled(
                highlight_rect,
                6.0 * scale,
                Color32::from_rgba_premultiplied(210, 140, 255, 100),
            );

            painter.rect_stroke(
                filled_rect.expand(0.5 * scale),
                6.5 * scale,
                egui::Stroke::new(
                    2.5 * scale,
                    Color32::from_rgba_premultiplied(180, 100, 220, 140),
                ),
                egui::StrokeKind::Outside,
            );

            let indicator_radius = 7.0 * scale;
            let indicator_pos = Pos2::new(filled_rect.max.x, bar_rect.center().y);

            painter.circle_filled(
                indicator_pos,
                4.0f32.mul_add(scale, indicator_radius),
                Color32::from_rgba_premultiplied(200, 120, 240, 40),
            );

            painter.circle_filled(
                indicator_pos,
                2.5f32.mul_add(scale, indicator_radius),
                Color32::from_rgba_premultiplied(190, 110, 230, 80),
            );

            painter.circle_filled(
                indicator_pos,
                indicator_radius,
                Color32::from_rgb(230, 180, 255),
            );

            painter.circle_filled(
                Pos2::new(
                    1.5f32.mul_add(-scale, indicator_pos.x),
                    1.5f32.mul_add(-scale, indicator_pos.y),
                ),
                indicator_radius * 0.5,
                Color32::from_rgba_premultiplied(255, 255, 255, 200),
            );
        }
    }
}
