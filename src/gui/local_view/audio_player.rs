use crate::gui::comms::command::CommsCommand;
use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{Color32, FontFamily, FontId};

fn format_time(milliseconds: u64) -> String {
    let total_seconds = milliseconds / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}

impl AkaiVisualizer {
    pub(crate) fn draw_audio_player(&self, ui: &egui::Ui, rect: Rect, scale: f32) {
        let player_height = 120.0 * scale;
        let margin = 15.0 * scale;

        let player_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + margin, rect.max.y - player_height - margin),
            Vec2::new(rect.width() - margin * 2.0, player_height),
        );

        ui.painter()
            .rect_filled(player_rect, 10.0 * scale, Color32::from_rgb(18, 18, 22));

        let glow_rect1 = player_rect.shrink(1.5 * scale);
        ui.painter().rect_stroke(
            glow_rect1,
            9.0 * scale,
            egui::Stroke::new(
                3.0 * scale,
                Color32::from_rgba_premultiplied(140, 70, 180, 80),
            ),
            egui::StrokeKind::Inside,
        );

        let glow_rect2 = player_rect.shrink(3.0 * scale);
        ui.painter().rect_stroke(
            glow_rect2,
            8.0 * scale,
            egui::Stroke::new(
                1.5 * scale,
                Color32::from_rgba_premultiplied(180, 100, 220, 40),
            ),
            egui::StrokeKind::Inside,
        );

        let (
            file_path,
            progress,
            elapsed_ms,
            total_ms,
            is_shuffled,
            _is_looped,
            is_muted,
            is_paused,
            is_solo,
            is_stop_all,
        ) = if let Ok(gui_data) = self.gui_data.lock()
            && let Some(playlist) = gui_data.data.current_playlist.clone()
        {
            let track = playlist.tracks[playlist.current_track as usize].clone();
            let track_length = track.track_length;

            let is_pause_on = gui_data.audio_player_states.pause_on;
            let is_stop_all_on = gui_data.audio_player_states.stop_all_on;

            let prog = if is_pause_on || is_stop_all_on {
                0.0
            } else if track_length > 0 {
                (gui_data.audio_player_states.local_elapsed as f32 / ((track_length * 1000) as f32))
                    .clamp(0.0, 1.0)
            } else {
                0.0
            };
            (
                track.file_path,
                prog,
                gui_data.audio_player_states.local_elapsed,
                track_length * 1000,
                gui_data.audio_player_states.shuffle_on,
                gui_data.audio_player_states.loop_on,
                gui_data.audio_player_states.mute_on,
                is_pause_on,
                gui_data.audio_player_states.solo_on,
                is_stop_all_on,
            )
        } else {
            (
                String::new(),
                0.0,
                0,
                0,
                false,
                false,
                false,
                false,
                false,
                false,
            )
        };

        let content_padding = 14.0 * scale;

        let button_size = Vec2::new(28.0 * scale, 24.0 * scale);
        let button_spacing = 8.0 * scale;
        let buttons_y = content_padding + player_rect.min.y;
        let buttons_start_x = player_rect.min.x + content_padding;

        let shuffle_rect = Rect::from_min_size(Pos2::new(buttons_start_x, buttons_y), button_size);

        let shuffle_color = if is_shuffled {
            Color32::from_rgb(180, 100, 220)
        } else {
            Color32::from_rgb(60, 60, 70)
        };

        ui.painter()
            .rect_filled(shuffle_rect, 4.0 * scale, shuffle_color);
        ui.painter().rect_stroke(
            shuffle_rect,
            4.0 * scale,
            egui::Stroke::new(1.5 * scale, Color32::from_rgb(100, 100, 110)),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            shuffle_rect.center(),
            egui::Align2::CENTER_CENTER,
            "🔀",
            FontId::proportional(12.0 * scale),
            Color32::from_rgb(240, 240, 250),
        );

        let shuffle_response = ui.interact(
            shuffle_rect,
            ui.id().with("shuffle_btn"),
            egui::Sense::click(),
        );
        if shuffle_response.clicked() {
            if let Ok(mut x) = self.gui_data.lock() {
                x.audio_player_states.shuffle_on = !x.audio_player_states.shuffle_on;
            }
            self.send_command_to_backend(CommsCommand::ShufflePressed {});
        }

        let mute_rect = Rect::from_min_size(
            Pos2::new(buttons_start_x + button_size.x + button_spacing, buttons_y),
            button_size,
        );

        let mute_color = if is_muted {
            Color32::from_rgb(200, 80, 80)
        } else {
            Color32::from_rgb(60, 60, 70)
        };

        ui.painter().rect_filled(mute_rect, 4.0 * scale, mute_color);
        ui.painter().rect_stroke(
            mute_rect,
            4.0 * scale,
            egui::Stroke::new(1.5 * scale, Color32::from_rgb(100, 100, 110)),
            egui::StrokeKind::Outside,
        );

        let mute_icon = if is_muted { "🔇" } else { "🔊" };
        ui.painter().text(
            mute_rect.center(),
            egui::Align2::CENTER_CENTER,
            mute_icon,
            FontId::proportional(12.0 * scale),
            Color32::from_rgb(240, 240, 250),
        );

        let mute_response = ui.interact(mute_rect, ui.id().with("mute_btn"), egui::Sense::click());
        if mute_response.clicked() {
            if let Ok(mut x) = self.gui_data.lock() {
                x.audio_player_states.mute_on = !x.audio_player_states.mute_on;
            }
            self.send_command_to_backend(CommsCommand::MutePressed {});
        }

        let solo_rect = Rect::from_min_size(
            Pos2::new(
                (button_size.x + button_spacing).mul_add(2.0, buttons_start_x),
                buttons_y,
            ),
            button_size,
        );

        let solo_color = if is_solo {
            Color32::from_rgb(100, 180, 220)
        } else {
            Color32::from_rgb(60, 60, 70)
        };

        ui.painter().rect_filled(solo_rect, 4.0 * scale, solo_color);
        ui.painter().rect_stroke(
            solo_rect,
            4.0 * scale,
            egui::Stroke::new(1.5 * scale, Color32::from_rgb(100, 100, 110)),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            solo_rect.center(),
            egui::Align2::CENTER_CENTER,
            "S",
            FontId::proportional(14.0 * scale),
            Color32::from_rgb(240, 240, 250),
        );

        let solo_response = ui.interact(solo_rect, ui.id().with("solo_btn"), egui::Sense::click());
        if solo_response.clicked() {
            if let Ok(mut x) = self.gui_data.lock() {
                x.audio_player_states.solo_on = !x.audio_player_states.solo_on;
            }
            self.send_command_to_backend(CommsCommand::SoloPressed {});
        }

        let stop_all_rect = Rect::from_min_size(
            Pos2::new(
                (button_size.x + button_spacing).mul_add(3.0, buttons_start_x),
                buttons_y,
            ),
            button_size,
        );

        let stop_all_color = if is_stop_all {
            Color32::from_rgb(220, 80, 80)
        } else {
            Color32::from_rgb(60, 60, 70)
        };

        ui.painter()
            .rect_filled(stop_all_rect, 4.0 * scale, stop_all_color);
        ui.painter().rect_stroke(
            stop_all_rect,
            4.0 * scale,
            egui::Stroke::new(1.5 * scale, Color32::from_rgb(100, 100, 110)),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            stop_all_rect.center(),
            egui::Align2::CENTER_CENTER,
            "⏹",
            FontId::proportional(14.0 * scale),
            Color32::from_rgb(240, 240, 250),
        );

        let stop_all_response = ui.interact(
            stop_all_rect,
            ui.id().with("stop_all_btn"),
            egui::Sense::click(),
        );
        if stop_all_response.clicked() {
            if let Ok(mut x) = self.gui_data.lock() {
                x.audio_player_states.stop_all_on = !x.audio_player_states.stop_all_on;
            }
            self.send_command_to_backend(CommsCommand::StopAllPressed {});
        }

        let title = std::path::Path::new(&file_path)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("No Track Playing")
            .to_string();

        let title_y = 8.0f32.mul_add(scale, buttons_y + button_size.y);

        ui.painter().text(
            Pos2::new(content_padding + player_rect.min.x + scale, title_y + scale),
            egui::Align2::LEFT_TOP,
            &title,
            FontId {
                size: 17.0 * scale,
                family: FontFamily::Name("Pixelify".into()),
            },
            Color32::from_rgba_premultiplied(0, 0, 0, 150),
        );

        ui.painter().text(
            Pos2::new(content_padding + player_rect.min.x, title_y),
            egui::Align2::LEFT_TOP,
            &title,
            FontId {
                size: 17.0 * scale,
                family: FontFamily::Name("Pixelify".into()),
            },
            Color32::from_rgb(245, 245, 250),
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

        if progress > 0.0 {
            let wave_height = 20.0 * scale;
            let wave_y = 32.0f32.mul_add(
                -scale,
                8.0f32.mul_add(
                    -scale,
                    12.0f32.mul_add(-scale, player_rect.max.y - content_padding),
                ) - wave_height,
            );
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

        let bar_margin = content_padding;
        let bar_height = 12.0 * scale;
        let bar_y = player_rect.max.y - bar_margin - bar_height;

        let bar_rect = Rect::from_min_size(
            Pos2::new(player_rect.min.x + bar_margin, bar_y),
            Vec2::new(player_rect.width() - bar_margin * 2.0, bar_height),
        );

        ui.painter()
            .rect_filled(bar_rect, 6.0 * scale, Color32::from_rgb(35, 35, 45));

        let shadow_rect =
            Rect::from_min_size(bar_rect.min, Vec2::new(bar_rect.width(), 3.0 * scale));
        ui.painter().rect_filled(
            shadow_rect,
            6.0 * scale,
            Color32::from_rgba_premultiplied(0, 0, 0, 100),
        );

        ui.painter().rect_stroke(
            bar_rect,
            6.0 * scale,
            egui::Stroke::new(1.5 * scale, Color32::from_rgb(55, 55, 65)),
            egui::StrokeKind::Outside,
        );

        if progress > 0.0 {
            let filled_width = bar_rect.width() * progress;
            let filled_rect =
                Rect::from_min_size(bar_rect.min, Vec2::new(filled_width, bar_rect.height()));

            ui.painter()
                .rect_filled(filled_rect, 6.0 * scale, Color32::from_rgb(150, 70, 190));

            let highlight_rect = Rect::from_min_size(
                filled_rect.min,
                Vec2::new(filled_rect.width(), filled_rect.height() * 0.45),
            );
            ui.painter().rect_filled(
                highlight_rect,
                6.0 * scale,
                Color32::from_rgba_premultiplied(210, 140, 255, 100),
            );

            ui.painter().rect_stroke(
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

            ui.painter().circle_filled(
                indicator_pos,
                4.0f32.mul_add(scale, indicator_radius),
                Color32::from_rgba_premultiplied(200, 120, 240, 40),
            );

            ui.painter().circle_filled(
                indicator_pos,
                2.5f32.mul_add(scale, indicator_radius),
                Color32::from_rgba_premultiplied(190, 110, 230, 80),
            );

            ui.painter().circle_filled(
                indicator_pos,
                indicator_radius,
                Color32::from_rgb(230, 180, 255),
            );

            ui.painter().circle_filled(
                Pos2::new(
                    1.5f32.mul_add(-scale, indicator_pos.x),
                    1.5f32.mul_add(-scale, indicator_pos.y),
                ),
                indicator_radius * 0.5,
                Color32::from_rgba_premultiplied(255, 255, 255, 200),
            );
        }

        let bottom_button_size = Vec2::new(32.0 * scale, 32.0 * scale);
        let bottom_buttons_y = bottom_button_size.y.mul_add(-0.5, bar_y) + bar_height * 0.5;
        let center_x = player_rect.center().x;

        let pause_rect = Rect::from_center_size(
            Pos2::new(
                4.0f32.mul_add(-scale, bottom_button_size.x.mul_add(-0.5, center_x)),
                bottom_button_size.y.mul_add(0.5, bottom_buttons_y),
            ),
            bottom_button_size,
        );

        ui.painter()
            .rect_filled(pause_rect, 4.0 * scale, Color32::from_rgb(60, 60, 70));
        ui.painter().rect_stroke(
            pause_rect,
            4.0 * scale,
            egui::Stroke::new(1.5 * scale, Color32::from_rgb(100, 100, 110)),
            egui::StrokeKind::Outside,
        );

        let pause_icon = if is_paused { "▶" } else { "⏸" };
        ui.painter().text(
            pause_rect.center(),
            egui::Align2::CENTER_CENTER,
            pause_icon,
            FontId::proportional(16.0 * scale),
            Color32::from_rgb(240, 240, 250),
        );

        let pause_response =
            ui.interact(pause_rect, ui.id().with("pause_btn"), egui::Sense::click());
        if pause_response.clicked() {
            if let Ok(mut x) = self.gui_data.lock() {
                x.audio_player_states.pause_on = !x.audio_player_states.pause_on;
            }
            self.send_command_to_backend(CommsCommand::PausePressed {});
        }

        let skip_rect = Rect::from_center_size(
            Pos2::new(
                4.0f32.mul_add(scale, bottom_button_size.x.mul_add(0.5, center_x)),
                bottom_button_size.y.mul_add(0.5, bottom_buttons_y),
            ),
            bottom_button_size,
        );

        ui.painter()
            .rect_filled(skip_rect, 4.0 * scale, Color32::from_rgb(60, 60, 70));
        ui.painter().rect_stroke(
            skip_rect,
            4.0 * scale,
            egui::Stroke::new(1.5 * scale, Color32::from_rgb(100, 100, 110)),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            skip_rect.center(),
            egui::Align2::CENTER_CENTER,
            "⏭",
            FontId::proportional(16.0 * scale),
            Color32::from_rgb(240, 240, 250),
        );

        let skip_response = ui.interact(skip_rect, ui.id().with("skip_btn"), egui::Sense::click());
        if skip_response.clicked() {
            self.send_command_to_backend(CommsCommand::SkipTrackPressed {});
        }
    }
}
