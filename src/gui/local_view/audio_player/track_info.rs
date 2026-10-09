use crate::gui::local_view::audio_player::utils::{draw_text_with_shadow, format_time};
use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{Color32, FontFamily, FontId};
use std::time::Duration;

impl AkaiVisualizer {
    // Track title and time display
    pub(crate) fn draw_track_info(
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        file_path: &str,
        elapsed_ms: Duration,
        total_ms: Duration,
    ) {
        let button_size = Vec2::new(28.0 * scale, 24.0 * scale);
        let buttons_y = content_padding + player_rect.min.y;
        let title_y = 8.0f32.mul_add(scale, buttons_y + button_size.y);

        let title = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("No Track Playing")
            .to_string();

        draw_text_with_shadow(
            ui.painter(),
            Pos2::new(content_padding + player_rect.min.x, title_y),
            egui::Align2::LEFT_TOP,
            &title,
            FontId {
                size: 17.0 * scale,
                family: FontFamily::Name("Pixelify".into()),
            },
            Color32::from_rgb(245, 245, 250),
            scale,
        );

        let time_text = format!("{} / {}", format_time(elapsed_ms), format_time(total_ms));
        ui.painter().text(
            Pos2::new(player_rect.max.x - content_padding, title_y),
            egui::Align2::RIGHT_TOP,
            &time_text,
            FontId {
                size: 13.0 * scale,
                family: FontFamily::Name("Pixelify".into()),
            },
            Color32::from_rgb(200, 160, 220),
        );
    }
}
