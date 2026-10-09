use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::Color32;

impl AkaiVisualizer {
    // Waveform visualizer
    pub(crate) fn draw_visualizer(
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        progress: f32,
    ) {
        let wave_height = 20.0 * scale;
        let wave_y = 32.0f32.mul_add(
            -scale,
            8.0f32.mul_add(
                -scale,
                12.0f32.mul_add(-scale, player_rect.max.y - content_padding),
            ),
        ) - wave_height;
        let num_bars = 50;
        let bar_width =
            (content_padding.mul_add(-2.0, player_rect.width()) / num_bars as f32) * 0.75;
        let wave_x_start = player_rect.min.x + content_padding;

        for i in 0..num_bars {
            let x = wave_x_start
                + (i as f32 * content_padding.mul_add(-2.0, player_rect.width()) / num_bars as f32);
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

            ui.painter().rect_filled(
                wave_rect,
                1.5 * scale,
                Color32::from_rgba_premultiplied(180, 100, 220, opacity),
            );

            if opacity > 100 {
                let highlight =
                    Rect::from_min_size(wave_rect.min, Vec2::new(bar_width, bar_height * 0.3));
                ui.painter().rect_filled(
                    highlight,
                    1.5 * scale,
                    Color32::from_rgba_premultiplied(220, 160, 255, opacity / 2),
                );
            }
        }
    }
}
