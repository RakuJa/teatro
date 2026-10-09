use crate::gui::local_view::audio_player_states::PlayerStatus;
use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::Color32;
use std::time::Duration;

impl AkaiVisualizer {
    pub(crate) fn draw_audio_player(&self, ui: &egui::Ui, rect: Rect, scale: f32) {
        let player_height = 120.0 * scale;
        let margin = 15.0 * scale;
        let content_padding = 14.0 * scale;

        let player_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + margin, rect.max.y - player_height - margin),
            Vec2::new(rect.width() - margin * 2.0, player_height),
        );

        Self::draw_player_background(ui, player_rect, scale);

        let (file_path, progress, elapsed_ms, total_ms, current_status) =
            self.get_audio_player_data();

        self.draw_control_buttons(ui, player_rect, content_padding, scale, current_status);
        Self::draw_track_info(
            ui,
            player_rect,
            content_padding,
            scale,
            &file_path,
            elapsed_ms,
            total_ms,
        );
        if progress > 0.0 {
            Self::draw_visualizer(ui, player_rect, content_padding, scale, progress);
        }
        self.draw_progress_bar(ui, player_rect, content_padding, scale, progress);
        self.draw_playback_buttons(ui, player_rect, scale, current_status);
        self.draw_playlist_panel(ui, player_rect, scale);
    }

    fn draw_player_background(ui: &egui::Ui, player_rect: Rect, scale: f32) {
        ui.painter()
            .rect_filled(player_rect, 10.0 * scale, Color32::from_rgb(18, 18, 22));

        ui.painter().rect_stroke(
            player_rect.shrink(1.5 * scale),
            9.0 * scale,
            egui::Stroke::new(
                3.0 * scale,
                Color32::from_rgba_premultiplied(140, 70, 180, 80),
            ),
            egui::StrokeKind::Inside,
        );

        ui.painter().rect_stroke(
            player_rect.shrink(3.0 * scale),
            8.0 * scale,
            egui::Stroke::new(
                1.5 * scale,
                Color32::from_rgba_premultiplied(180, 100, 220, 40),
            ),
            egui::StrokeKind::Inside,
        );
    }

    fn get_audio_player_data(&self) -> (String, f32, Duration, Duration, PlayerStatus) {
        if let Ok(gui_data) = self.gui_data.lock()
            && let Some(playlist) = gui_data.data.current_playlist.clone()
            && let Some(track) = playlist.tracks.get(playlist.current_track as usize)
        {
            let prog = if gui_data.player_info.status.is_music_playable() {
                0.
            } else if !track.track_length.is_zero() {
                (gui_data.player_info.local_elapsed.as_secs_f64()
                    / track.track_length.as_secs_f64())
                .clamp(0., 1.)
            } else {
                0.
            };
            (
                track.file_path.clone(),
                prog as f32,
                gui_data.player_info.local_elapsed,
                track.track_length,
                gui_data.player_info.status,
            )
        } else {
            (
                String::new(),
                0.0,
                Duration::default(),
                Duration::default(),
                PlayerStatus::default(),
            )
        }
    }
}
