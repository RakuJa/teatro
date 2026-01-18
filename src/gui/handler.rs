use crate::os_explorer::explorer::{get_album_name_from_folder_in_path, map_to_indexed_vec};
use crate::states::visualizer::AkaiData;
use eframe::egui;
use egui::{Color32, Pos2, Rect, Vec2};

pub struct AkaiVisualizer {
    pub data: AkaiData,
}

impl eframe::App for AkaiVisualizer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let _ = self.update_pad_labels(self.data.music_folder.clone().as_str());
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Teatro - Akai APC Key 25 Controller Visualizer");
            ui.add_space(10.0);

            // Calculate scale based on available space
            let available_size = ui.available_size();
            let base_width = 1150.0;
            let base_height = 560.0;
            let scale = (available_size.x / base_width)
                .min(available_size.y / base_height)
                .min(2.0);

            let scaled_width = base_width * scale;
            let scaled_height = base_height * scale;

            // Main controller area
            let (response, painter) =
                ui.allocate_painter(Vec2::new(scaled_width, scaled_height), egui::Sense::hover());

            let rect = response.rect;
            painter.rect_filled(rect, 5.0 * scale, Color32::from_rgb(30, 30, 35));

            // Draw components with scale
            self.draw_pads(&painter, rect, scale);
            self.draw_knobs(&painter, rect, ui, scale);
            Self::draw_soft_keys(&painter, rect, scale);
            Self::draw_control_buttons(&painter, rect, scale);
            Self::draw_keyboard(&painter, rect, scale);

            ui.add_space(5.0);
            ui.label(format!("Music folder: {} ", self.data.music_folder));
        });
    }
}

impl AkaiVisualizer {
    fn update_pad_labels(&mut self, music_folder: &str) -> anyhow::Result<()> {
        let albums = map_to_indexed_vec(get_album_name_from_folder_in_path(music_folder)?);
        for (i, album) in albums.into_iter().enumerate() {
            if let Some(name) = album {
                self.data.pad_labels[i] = name;
            }
        }
        Ok(())
    }
    fn draw_pads(&self, painter: &egui::Painter, rect: Rect, scale: f32) {
        let pad_size = 45.0 * scale;
        let pad_spacing = 8.0 * scale;
        let start_x = 20.0f32.mul_add(scale, rect.min.x);
        let start_y = 120.0f32.mul_add(scale, rect.min.y);

        for row in 0..5 {
            for col in 0..8 {
                // Bottom-left is 0, so we need to reverse the row indexing
                let idx = (4 - row) * 8 + col;
                let x = f32::from(col).mul_add(pad_size + pad_spacing, start_x);
                let y = f32::from(row).mul_add(pad_size + pad_spacing, start_y);

                let pad_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(pad_size, pad_size));

                // Draw pad background
                let color = if self.data.last_pad_pressed.is_some_and(|x| x == idx) {
                    Color32::from_rgb(52, 21, 57)
                } else {
                    Color32::from_rgb(80, 180, 100)
                };

                painter.rect_filled(pad_rect, 3.0 * scale, color);
                painter.rect_stroke(
                    pad_rect,
                    3.0 * scale,
                    egui::Stroke::new(1.5 * scale, Color32::BLACK),
                    egui::StrokeKind::Outside,
                );

                // Draw pad number
                painter.text(
                    Pos2::new(5.0f32.mul_add(scale, x), 5.0f32.mul_add(scale, y)),
                    egui::Align2::LEFT_TOP,
                    format!("{idx}"),
                    egui::FontId::proportional(10.0 * scale),
                    Color32::WHITE,
                );

                // Draw label if exists
                if !self.data.pad_labels[idx as usize].is_empty() {
                    painter.text(
                        pad_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &self.data.pad_labels[idx as usize],
                        egui::FontId::proportional(11.0 * scale),
                        Color32::WHITE,
                    );
                }
            }
        }
    }

    fn draw_knobs(&self, painter: &egui::Painter, rect: Rect, _ui: &mut egui::Ui, scale: f32) {
        let knob_radius = 20.0 * scale;
        let knob_spacing = 55.0 * scale;
        let start_x = 450.0f32.mul_add(scale, rect.min.x);
        let start_y = 30.0f32.mul_add(scale, rect.min.y);

        for i in 0u8..8 {
            let x = f32::from(i).mul_add(knob_spacing, start_x);
            let y = start_y;
            let center = Pos2::new(x, y);

            // Knob background
            painter.circle_filled(center, knob_radius, Color32::from_rgb(60, 60, 70));
            painter.circle_stroke(
                center,
                knob_radius,
                egui::Stroke::new(2.0 * scale, Color32::from_rgb(100, 100, 110)),
            );

            // Knob indicator (value position)
            let angle = -2.4f32 + self.data.knob_values.get(&(i + 1)).unwrap_or(&0f32) * 4.8; // -2.4 to 2.4 radians
            let indicator_len = 5.0f32.mul_add(-scale, knob_radius);
            let end = Pos2::new(
                angle.cos().mul_add(indicator_len, x),
                angle.sin().mul_add(indicator_len, y),
            );
            painter.line_segment(
                [center, end],
                egui::Stroke::new(3.0 * scale, Color32::WHITE),
            );

            // Knob label
            painter.text(
                Pos2::new(x, 15.0f32.mul_add(scale, y + knob_radius)),
                egui::Align2::CENTER_TOP,
                format!("K{}", i + 1),
                egui::FontId::proportional(10.0 * scale),
                Color32::LIGHT_GRAY,
            );
        }
    }

    fn draw_soft_keys(painter: &egui::Painter, rect: Rect, scale: f32) {
        let key_width = 30.0 * scale;
        let key_height = 20.0 * scale;
        let key_spacing = 55.0 * scale;
        let start_x = 435.0f32.mul_add(scale, rect.min.x);
        let start_y = 85.0f32.mul_add(scale, rect.min.y);

        for i in 0..8 {
            let x = (i as f32).mul_add(key_spacing, start_x);
            let y = start_y;

            let key_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(key_width, key_height));

            let color = Color32::from_rgb(100, 100, 110);

            painter.rect_filled(key_rect, 2.0 * scale, color);
            painter.rect_stroke(
                key_rect,
                2.0 * scale,
                egui::Stroke::new(1.0 * scale, Color32::BLACK),
                egui::StrokeKind::Outside,
            );

            painter.text(
                key_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("S{}", i + 1),
                egui::FontId::proportional(9.0 * scale),
                Color32::WHITE,
            );
        }
    }

    fn draw_control_buttons(painter: &egui::Painter, rect: Rect, scale: f32) {
        let btn_size = 25.0 * scale;
        let start_x = 1000.0f32.mul_add(scale, rect.min.x);
        let start_y = 30.0f32.mul_add(scale, rect.min.y);

        // Arrow buttons
        let arrows = [
            ("↑", 0.0, -30.0 * scale),
            ("↓", 0.0, 0.0),
            ("←", -30.0 * scale, 0.0),
            ("→", 30.0 * scale, 0.0),
        ];

        for (symbol, dx, dy) in arrows {
            let btn_rect = Rect::from_min_size(
                Pos2::new(start_x + dx, start_y + dy),
                Vec2::new(btn_size, btn_size),
            );
            painter.rect_filled(btn_rect, 3.0 * scale, Color32::from_rgb(70, 70, 80));
            painter.rect_stroke(
                btn_rect,
                3.0 * scale,
                egui::Stroke::new(1.0 * scale, Color32::BLACK),
                egui::StrokeKind::Outside,
            );
            painter.text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                symbol,
                egui::FontId::proportional(16.0 * scale),
                Color32::WHITE,
            );
        }

        // Other control buttons
        let controls = [
            ("SHIFT", 0.0, 50.0 * scale),
            ("STOP", 0.0, 80.0 * scale),
            ("PLAY", 0.0, 110.0 * scale),
            ("REC", 0.0, 140.0 * scale),
        ];

        for (label, dx, dy) in controls {
            let btn_rect = Rect::from_min_size(
                Pos2::new(15.0f32.mul_add(-scale, start_x + dx), start_y + dy),
                Vec2::new(55.0 * scale, btn_size),
            );
            painter.rect_filled(btn_rect, 3.0 * scale, Color32::from_rgb(70, 70, 80));
            painter.rect_stroke(
                btn_rect,
                3.0 * scale,
                egui::Stroke::new(1.0 * scale, Color32::BLACK),
                egui::StrokeKind::Outside,
            );
            painter.text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(10.0 * scale),
                Color32::WHITE,
            );
        }
    }

    fn draw_keyboard(painter: &egui::Painter, rect: Rect, scale: f32) {
        let white_key_width = 35.0 * scale;
        let white_key_height = 120.0 * scale;
        let black_key_width = 22.0 * scale;
        let black_key_height = 75.0 * scale;
        let start_x = 20.0f32.mul_add(scale, rect.min.x);
        let start_y = 420.0f32.mul_add(scale, rect.min.y);

        // Pattern: W B W B W W B W B W B W (C to C = 2 octaves)
        let pattern = [
            true, false, true, false, true, true, false, true, false, true, false, true,
        ];

        // Draw white keys first
        let mut white_idx = 0;
        for i in 0..25 {
            let is_white = pattern[i % 12];
            if is_white {
                let x = (white_idx as f32).mul_add(white_key_width, start_x);
                let key_rect = Rect::from_min_size(
                    Pos2::new(x, start_y),
                    Vec2::new(white_key_width, white_key_height),
                );

                painter.rect_filled(key_rect, 2.0 * scale, Color32::WHITE);
                painter.rect_stroke(
                    key_rect,
                    2.0 * scale,
                    egui::Stroke::new(2.0 * scale, Color32::BLACK),
                    egui::StrokeKind::Outside,
                );
                white_idx += 1;
            }
        }

        // Draw black keys on top
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

                painter.rect_filled(key_rect, 2.0 * scale, Color32::BLACK);
                painter.rect_stroke(
                    key_rect,
                    2.0 * scale,
                    egui::Stroke::new(1.0 * scale, Color32::DARK_GRAY),
                    egui::StrokeKind::Outside,
                );
            }
        }
    }
}
