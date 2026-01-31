use crate::gui::comms::command::CommsCommand;
use crate::gui::local_view::audio_player_states::PlayerStatus;
use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{Color32, FontFamily, FontId};

fn format_time(milliseconds: u64) -> String {
    let total_seconds = milliseconds / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}

struct ButtonStyle {
    size: Vec2,
    bg_color: Color32,
    border_color: Color32,
    border_width: f32,
    corner_radius: f32,
}

impl ButtonStyle {
    fn new(scale: f32) -> Self {
        Self {
            size: Vec2::new(28.0 * scale, 24.0 * scale),
            bg_color: Color32::from_rgb(60, 60, 70),
            border_color: Color32::from_rgb(100, 100, 110),
            border_width: 1.5 * scale,
            corner_radius: 4.0 * scale,
        }
    }

    const fn active_color(mut self, color: Color32) -> Self {
        self.bg_color = color;
        self
    }

    const fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
}

fn draw_button(
    ui: &egui::Ui,
    rect: Rect,
    style: &ButtonStyle,
    icon: &str,
    icon_size: f32,
    id: &str,
) -> egui::Response {
    ui.painter()
        .rect_filled(rect, style.corner_radius, style.bg_color);
    ui.painter().rect_stroke(
        rect,
        style.corner_radius,
        egui::Stroke::new(style.border_width, style.border_color),
        egui::StrokeKind::Outside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        FontId::proportional(icon_size),
        Color32::from_rgb(240, 240, 250),
    );

    ui.interact(rect, ui.id().with(id), egui::Sense::click())
}

fn draw_text_with_shadow(
    painter: &egui::Painter,
    pos: Pos2,
    align: egui::Align2,
    text: &str,
    font: FontId,
    color: Color32,
    shadow_offset: f32,
) {
    // Shadow
    painter.text(
        Pos2::new(pos.x + shadow_offset, pos.y + shadow_offset),
        align,
        text,
        font.clone(),
        Color32::from_rgba_premultiplied(0, 0, 0, 150),
    );
    painter.text(pos, align, text, font, color);
}

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

        let (file_path, progress, elapsed_ms, total_ms, current_status) = self.get_player_data();

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
        Self::draw_progress_bar(ui, player_rect, content_padding, scale, progress);
        self.draw_playback_buttons(ui, player_rect, scale, current_status);
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

    fn get_player_data(&self) -> (String, f32, u64, u64, PlayerStatus) {
        if let Ok(gui_data) = self.gui_data.lock()
            && let Some(playlist) = gui_data.data.current_playlist.clone()
            && let Some(track) = playlist.tracks.get(playlist.current_track as usize)
        {
            let track_length = track.track_length;

            let prog = if gui_data.player_info.status.is_music_playable() {
                0.0
            } else if track_length > 0 {
                (gui_data.player_info.local_elapsed as f32 / ((track_length * 1000) as f32))
                    .clamp(0.0, 1.0)
            } else {
                0.0
            };
            (
                track.file_path.clone(),
                prog,
                gui_data.player_info.local_elapsed,
                track_length * 1000,
                gui_data.player_info.status,
            )
        } else {
            (String::new(), 0.0, 0, 0, PlayerStatus::default())
        }
    }

    fn draw_control_buttons(
        &self,
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        current_status: PlayerStatus,
    ) {
        let button_size = Vec2::new(28.0 * scale, 24.0 * scale);
        let button_spacing = 8.0 * scale;
        let buttons_y = content_padding + player_rect.min.y;
        let buttons_start_x = player_rect.min.x + content_padding;

        let buttons = [
            (
                "shuffle",
                "🔀",
                current_status.is_shuffle_requested(),
                Color32::from_rgb(180, 100, 220),
                PlayerStatus::SHUFFLE,
                CommsCommand::ShufflePressed {},
            ),
            (
                "mute",
                if current_status.is_music_muted() {
                    "🔇"
                } else {
                    "🔊"
                },
                current_status.is_music_muted(),
                Color32::from_rgb(200, 80, 80),
                PlayerStatus::MUTE_ALL,
                CommsCommand::MutePressed {},
            ),
            (
                "solo",
                "S",
                current_status.is_sound_muted(),
                Color32::from_rgb(100, 180, 220),
                PlayerStatus::SOLO_MUSIC,
                CommsCommand::SoloPressed {},
            ),
            (
                "stop_all",
                "⏹",
                current_status.is_everything_stopped(),
                Color32::from_rgb(220, 80, 80),
                PlayerStatus::STOP_ALL,
                CommsCommand::StopAllPressed {},
            ),
        ];

        for (i, (id, icon, is_active, active_color, status_flag, command)) in
            buttons.iter().enumerate()
        {
            let x = (button_size.x + button_spacing).mul_add(i as f32, buttons_start_x);
            let rect = Rect::from_min_size(Pos2::new(x, buttons_y), button_size);

            let style = ButtonStyle::new(scale).active_color(if *is_active {
                *active_color
            } else {
                Color32::from_rgb(60, 60, 70)
            });

            let icon_size = if *id == "solo" { 14.0 } else { 12.0 } * scale;
            let response = draw_button(ui, rect, &style, icon, icon_size, id);

            if response.clicked()
                && matches!(
                    self.gui_data
                        .lock()
                        .map(|mut x| x.player_info.status.toggle(*status_flag)),
                    Ok(())
                )
            {
                self.send_command_to_backend(*command);
            }
        }
    }

    // Track title and time display
    fn draw_track_info(
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        file_path: &str,
        elapsed_ms: u64,
        total_ms: u64,
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

    // Waveform visualizer
    fn draw_visualizer(
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

    // Progress bar with indicator
    fn draw_progress_bar(
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        progress: f32,
    ) {
        let bar_height = 12.0 * scale;
        let bar_y = player_rect.max.y - content_padding - bar_height;

        let bar_rect = Rect::from_min_size(
            Pos2::new(player_rect.min.x + content_padding, bar_y),
            Vec2::new(
                content_padding.mul_add(-2.0, player_rect.width()),
                bar_height,
            ),
        );

        // Background
        ui.painter()
            .rect_filled(bar_rect, 6.0 * scale, Color32::from_rgb(35, 35, 45));

        // Shadow
        let shadow_rect =
            Rect::from_min_size(bar_rect.min, Vec2::new(bar_rect.width(), 3.0 * scale));
        ui.painter().rect_filled(
            shadow_rect,
            6.0 * scale,
            Color32::from_rgba_premultiplied(0, 0, 0, 100),
        );

        // Border
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

            // Filled portion
            ui.painter()
                .rect_filled(filled_rect, 6.0 * scale, Color32::from_rgb(150, 70, 190));

            // Highlight
            let highlight_rect = Rect::from_min_size(
                filled_rect.min,
                Vec2::new(filled_rect.width(), filled_rect.height() * 0.45),
            );
            ui.painter().rect_filled(
                highlight_rect,
                6.0 * scale,
                Color32::from_rgba_premultiplied(210, 140, 255, 100),
            );

            // Glow
            ui.painter().rect_stroke(
                filled_rect.expand(0.5 * scale),
                6.5 * scale,
                egui::Stroke::new(
                    2.5 * scale,
                    Color32::from_rgba_premultiplied(180, 100, 220, 140),
                ),
                egui::StrokeKind::Outside,
            );

            // Position indicator
            Self::draw_position_indicator(ui, filled_rect.max.x, bar_rect.center().y, scale);
        }
    }

    // Position indicator dot
    fn draw_position_indicator(ui: &egui::Ui, x: f32, y: f32, scale: f32) {
        let indicator_radius = 7.0 * scale;
        let pos = Pos2::new(x, y);

        // Outer glow layers
        ui.painter().circle_filled(
            pos,
            4.0f32.mul_add(scale, indicator_radius),
            Color32::from_rgba_premultiplied(200, 120, 240, 40),
        );
        ui.painter().circle_filled(
            pos,
            2.5f32.mul_add(scale, indicator_radius),
            Color32::from_rgba_premultiplied(190, 110, 230, 80),
        );

        // Main circle
        ui.painter()
            .circle_filled(pos, indicator_radius, Color32::from_rgb(230, 180, 255));

        // Highlight
        ui.painter().circle_filled(
            Pos2::new(1.5f32.mul_add(-scale, pos.x), 1.5f32.mul_add(-scale, pos.y)),
            indicator_radius * 0.5,
            Color32::from_rgba_premultiplied(255, 255, 255, 200),
        );
    }

    // Playback buttons (pause, skip)
    fn draw_playback_buttons(
        &self,
        ui: &egui::Ui,
        player_rect: Rect,
        scale: f32,
        current_status: PlayerStatus,
    ) {
        let button_size = Vec2::new(32.0 * scale, 32.0 * scale);
        let bar_height = 12.0 * scale;
        let content_padding = 14.0 * scale;
        let bar_y = player_rect.max.y - content_padding - bar_height;
        let buttons_y = bar_y - button_size.y / 2.0 + bar_height / 2.0;
        let center_x = player_rect.center().x;

        let style = ButtonStyle::new(scale).with_size(button_size);

        // Pause button
        let pause_rect = Rect::from_center_size(
            Pos2::new(
                4.0f32.mul_add(-scale, center_x - button_size.x / 2.0),
                buttons_y + button_size.y / 2.0,
            ),
            button_size,
        );
        let pause_icon = if current_status.is_music_paused() {
            "▶"
        } else {
            "⏸"
        };
        let pause_response = draw_button(
            ui,
            pause_rect,
            &style,
            pause_icon,
            16.0 * scale,
            "pause_btn",
        );

        if pause_response.clicked()
            && matches!(
                self.gui_data
                    .lock()
                    .map(|mut x| x.player_info.status.toggle(PlayerStatus::PAUSE_MUSIC)),
                Ok(())
            )
        {
            self.send_command_to_backend(CommsCommand::PausePressed {});
        }

        // Skip button
        let skip_rect = Rect::from_center_size(
            Pos2::new(
                4.0f32.mul_add(scale, center_x + button_size.x / 2.0),
                buttons_y + button_size.y / 2.0,
            ),
            button_size,
        );
        let skip_response = draw_button(ui, skip_rect, &style, "⏭", 16.0 * scale, "skip_btn");

        if skip_response.clicked() {
            self.send_command_to_backend(CommsCommand::SkipTrackPressed {});
        }
    }
}
