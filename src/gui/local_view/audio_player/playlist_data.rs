use crate::gui::comms::command::CommsCommand;
use crate::gui::local_view::audio_player::button_style::{ButtonStyle, draw_button};
use crate::gui::local_view::audio_player::utils::{
    format_time, is_playlist_open, set_playlist_open, title_from_path,
};
use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{Color32, FontFamily, FontId};
use std::time::Duration;

impl AkaiVisualizer {
    pub fn draw_playlist_toggle(
        &self,
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        button_index: usize,
    ) {
        let button_size = Vec2::new(28.0 * scale, 24.0 * scale);
        let button_spacing = 8.0 * scale;
        let x = (button_size.x + button_spacing)
            .mul_add(button_index as f32, player_rect.min.x + content_padding);
        let rect = Rect::from_min_size(
            Pos2::new(x, player_rect.min.y + content_padding),
            button_size,
        );

        let open = is_playlist_open(ui.ctx());
        let style = ButtonStyle::new(scale).active_color(if open {
            Color32::from_rgb(120, 200, 140)
        } else {
            Color32::from_rgb(60, 60, 70)
        });

        let response = draw_button(ui, rect, &style, "☰", 13.0 * scale, "playlist_btn");
        if response.clicked() {
            set_playlist_open(ui.ctx(), !open);
            // Force a re-scroll to the current track whenever the panel is (re)opened
            ui.ctx()
                .data_mut(|d| d.remove::<usize>(egui::Id::new("akai_playlist_scrolled")));
        }
    }

    /// Returns (tracks as (`title`, `length_in_seconds`), `index` of current track)
    fn get_playlist_view(&self) -> Option<(Vec<(String, Duration)>, usize)> {
        let playlist = self
            .gui_data
            .lock()
            .ok()?
            .data
            .current_playlist
            .as_ref()?
            .clone();
        let tracks = playlist
            .tracks
            .iter()
            .map(|t| (title_from_path(&t.file_path), t.track_length))
            .collect();
        Some((tracks, playlist.current_track as usize))
    }

    /// Floating panel that sits right above the player and lists every track in the album.
    pub(crate) fn draw_playlist_panel(&self, ui: &egui::Ui, player_rect: Rect, scale: f32) {
        let ctx = ui.ctx().clone();
        if !is_playlist_open(&ctx) {
            return;
        }
        let Some((tracks, current)) = self.get_playlist_view() else {
            return;
        };

        let row_h = 26.0 * scale;
        let max_list_h = 240.0 * scale;
        let pad = 10.0 * scale;
        let gap = 8.0 * scale;
        let scrolled_id = egui::Id::new("akai_playlist_scrolled");
        let last_scrolled: Option<usize> = ctx.data(|d| d.get_temp(scrolled_id));
        let mut clicked_track: Option<usize> = None;

        egui::Area::new(egui::Id::new("akai_playlist_area"))
            .order(egui::Order::Foreground)
            .pivot(egui::Align2::LEFT_BOTTOM)
            .fixed_pos(Pos2::new(player_rect.min.x, player_rect.min.y - gap))
            .show(&ctx, |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgb(18, 18, 22))
                    .stroke(egui::Stroke::new(
                        1.5 * scale,
                        Color32::from_rgba_premultiplied(180, 100, 220, 90),
                    ))
                    .corner_radius(10.0 * scale)
                    .inner_margin(egui::Margin::same(pad as i8))
                    .show(ui, |ui| {
                        ui.set_width(2.0f32.mul_add(-pad, player_rect.width()));

                        ui.label(
                            egui::RichText::new(format!("Album tracks ({})", tracks.len()))
                                .font(FontId {
                                    size: 14.0 * scale,
                                    family: FontFamily::Name("Pixelify".into()),
                                })
                                .color(Color32::from_rgb(200, 160, 220)),
                        );
                        ui.add_space(4.0 * scale);

                        egui::ScrollArea::vertical()
                            .max_height(max_list_h)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                for (i, (title, len_secs)) in tracks.iter().enumerate() {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        Vec2::new(ui.available_width(), row_h),
                                        egui::Sense::click(),
                                    );
                                    let is_current = i == current;

                                    let bg = if is_current {
                                        Some(Color32::from_rgba_premultiplied(150, 70, 190, 110))
                                    } else if resp.hovered() {
                                        Some(Color32::from_rgba_premultiplied(255, 255, 255, 20))
                                    } else {
                                        None
                                    };
                                    if let Some(bg) = bg {
                                        ui.painter().rect_filled(rect, 4.0 * scale, bg);
                                    }

                                    // Clip so long titles never spill over the time column
                                    let painter = ui.painter().with_clip_rect(rect);
                                    let text_color = if is_current {
                                        Color32::from_rgb(245, 235, 255)
                                    } else {
                                        Color32::from_rgb(190, 190, 200)
                                    };
                                    let font = FontId {
                                        size: 13.0 * scale,
                                        family: FontFamily::Name("Pixelify".into()),
                                    };

                                    let marker = if is_current {
                                        ">".to_string()
                                    } else {
                                        format!("{:02}", i + 1)
                                    };
                                    painter.text(
                                        Pos2::new(
                                            8.0f32.mul_add(scale, rect.min.x),
                                            rect.center().y,
                                        ),
                                        egui::Align2::LEFT_CENTER,
                                        marker,
                                        font.clone(),
                                        Color32::from_rgb(200, 160, 220),
                                    );
                                    // Time on the right
                                    painter.text(
                                        Pos2::new(
                                            8.0f32.mul_add(-scale, rect.max.x),
                                            rect.center().y,
                                        ),
                                        egui::Align2::RIGHT_CENTER,
                                        format_time(*len_secs),
                                        font.clone(),
                                        Color32::from_rgb(200, 160, 220),
                                    );
                                    // Title (leaves room for the time column)
                                    let title_painter =
                                        ui.painter().with_clip_rect(Rect::from_min_max(
                                            rect.min,
                                            Pos2::new(
                                                60.0f32.mul_add(-scale, rect.max.x),
                                                rect.max.y,
                                            ),
                                        ));
                                    title_painter.text(
                                        Pos2::new(
                                            36.0f32.mul_add(scale, rect.min.x),
                                            rect.center().y,
                                        ),
                                        egui::Align2::LEFT_CENTER,
                                        title,
                                        font,
                                        text_color,
                                    );

                                    // Auto-scroll to the playing track (once per track change)
                                    if is_current && last_scrolled != Some(current) {
                                        resp.scroll_to_me(Some(egui::Align::Center));
                                    }

                                    if resp.clicked() {
                                        clicked_track = Some(i);
                                    }
                                }
                            });
                    });
            });

        ctx.data_mut(|d| d.insert_temp(scrolled_id, current));

        if let Some(index) = clicked_track {
            self.send_command_to_backend(CommsCommand::PlayTrackPressed { index });
        }
    }
}
